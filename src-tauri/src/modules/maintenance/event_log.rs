use crate::models::event_log_entry::EventLogEntry;

const CHANNELS: [&str; 2] = ["System", "Application"];
const MAX_EVENTS_PER_CHANNEL: usize = 25;

pub fn get_critical_events_last_24h() -> Result<Vec<EventLogEntry>, String> {
    let mut entries = Vec::new();

    for channel in CHANNELS {
        let mut channel_entries = query_channel(channel)?;
        entries.append(&mut channel_entries);
    }

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(MAX_EVENTS_PER_CHANNEL * CHANNELS.len());

    Ok(entries)
}

#[cfg(windows)]
fn query_channel(channel: &str) -> Result<Vec<EventLogEntry>, String> {
    use std::ffi::c_void;

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;
    use windows::Win32::System::EventLog::{
        EvtClose, EvtNext, EvtQuery, EvtQueryChannelPath, EvtQueryReverseDirection, EvtRender,
        EvtRenderEventXml, EVT_HANDLE,
    };

    let xpath = "*[System[(Level=1 or Level=2) and TimeCreated[timediff(@SystemTime) <= 86400000]]]";
    let channel_w = encode_wide(channel);
    let xpath_w = encode_wide(xpath);

    let query_handle = unsafe {
        EvtQuery(
            None,
            PCWSTR(channel_w.as_ptr()),
            PCWSTR(xpath_w.as_ptr()),
            EvtQueryChannelPath.0 | EvtQueryReverseDirection.0,
        )
    }
    .map_err(|e| format!("EvtQuery failed for {channel}: {e}"))?;

    let mut results = Vec::new();

    loop {
        let mut event_handles = [0isize; 1];
        let mut returned = 0u32;

        let next_result = unsafe {
            EvtNext(
                query_handle,
                &mut event_handles,
                1000,
                0,
                &mut returned,
            )
        };

        if next_result.is_err() {
            let err = next_result.unwrap_err();
            if err.code() == ERROR_NO_MORE_ITEMS.to_hresult() {
                break;
            }
            return Err(format!("EvtNext failed for {channel}: {err}"));
        }

        if returned == 0 {
            break;
        }

        let event = EVT_HANDLE(event_handles[0]);
        let xml = render_event_xml(event)?;
        if let Some(entry) = parse_event_xml(channel, &xml) {
            results.push(entry);
            if results.len() >= MAX_EVENTS_PER_CHANNEL {
                unsafe {
                    let _ = EvtClose(event);
                }
                break;
            }
        }

        unsafe {
            let _ = EvtClose(event);
        }
    }

    unsafe {
        let _ = EvtClose(query_handle);
    }

    Ok(results)
}

#[cfg(windows)]
fn render_event_xml(event: windows::Win32::System::EventLog::EVT_HANDLE) -> Result<String, String> {
    use std::ffi::c_void;

    use windows::Win32::System::EventLog::{EvtRender, EvtRenderEventXml};

    let mut buffer_used = 0u32;
    let mut property_count = 0u32;

    unsafe {
        let _ = EvtRender(
            None,
            event,
            EvtRenderEventXml.0,
            0,
            None,
            &mut buffer_used,
            &mut property_count,
        );
    }

    if buffer_used == 0 {
        return Err("EvtRender returned empty buffer size".into());
    }

    let mut buffer = vec![0u16; (buffer_used as usize / 2) + 1];

    unsafe {
        EvtRender(
            None,
            event,
            EvtRenderEventXml.0,
            buffer_used,
            Some(buffer.as_mut_ptr() as *mut c_void),
            &mut buffer_used,
            &mut property_count,
        )
        .map_err(|e| format!("EvtRender failed: {e}"))?;
    }

    Ok(String::from_utf16_lossy(&buffer))
}

#[cfg(windows)]
fn parse_event_xml(channel: &str, xml: &str) -> Option<EventLogEntry> {
    let provider = extract_xml_attribute(xml, "Provider", "Name")
        .unwrap_or_else(|| "Unknown".to_string());
    let event_id = extract_xml_text(xml, "EventID")
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let level_code = extract_xml_text(xml, "Level").unwrap_or_default();
    let timestamp = extract_xml_attribute(xml, "TimeCreated", "SystemTime")
        .unwrap_or_else(|| "Unknown".to_string());
    let message = extract_event_message(xml);

    Some(EventLogEntry {
        channel: channel.to_string(),
        level: map_event_level(&level_code),
        provider,
        event_id,
        message,
        timestamp,
    })
}

#[cfg(windows)]
fn extract_event_message(xml: &str) -> String {
    if let Some(data) = extract_xml_text(xml, "Data") {
        if !data.is_empty() {
            return truncate_message(&data);
        }
    }

    truncate_message(xml)
}

#[cfg(windows)]
fn extract_xml_attribute(xml: &str, tag: &str, attribute: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let slice = &xml[start..];
    let attr_needle = format!("{attribute}='");
    let attr_start = slice.find(&attr_needle)? + attr_needle.len();
    let attr_end = slice[attr_start..].find('\'')? + attr_start;
    Some(slice[attr_start..attr_end].to_string())
}

#[cfg(windows)]
fn extract_xml_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].trim().to_string())
}

#[cfg(windows)]
fn map_event_level(level: &str) -> String {
    match level {
        "1" => "Critical".to_string(),
        "2" => "Error".to_string(),
        _ => format!("Level {level}"),
    }
}

#[cfg(windows)]
fn truncate_message(message: &str) -> String {
    const MAX_LEN: usize = 240;
    let collapsed = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= MAX_LEN {
        collapsed
    } else {
        format!("{}…", &collapsed[..MAX_LEN])
    }
}

#[cfg(windows)]
fn encode_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(not(windows))]
fn query_channel(_channel: &str) -> Result<Vec<EventLogEntry>, String> {
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn get_critical_events_last_24h() -> Result<Vec<EventLogEntry>, String> {
    Err("Event log queries are only available on Windows".into())
}
