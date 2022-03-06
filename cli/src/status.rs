use log::{debug, info, warn, error};
use sysinfo::{ComponentExt, System, SystemExt};

enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

struct ReportMessage {
    level: LogLevel,
    content: String,
}

pub fn full_report() {
    print_report(sys_report());
}

fn print_report(messages: Vec<ReportMessage>) {
    for msg in &messages {
        match &msg.level {
            LogLevel::Debug => {
                debug!("{}", &msg.content)
            }
            LogLevel::Info => {
                info!("{}", &msg.content)
            },
            LogLevel::Warn => {
                warn!("{}", &msg.content)
            },
            _ => {
                error!("{}", &msg.content)
            }
        }
    }
}

fn sys_report() -> Vec<ReportMessage> {
    let mut msgs: Vec<ReportMessage> = vec![];

    // Please note that we use "new_all" to ensure that all list of
    // components, network interfaces, disks and users are already
    // filled!
    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    // Display system information:
    let system_info = [
        ("System host name:        ", sys.host_name()),
        ("System name:             ", sys.name()),
        ("System kernel version:   ", sys.kernel_version()),
        ("System OS version:       ", sys.os_version()),
    ];
    for info in system_info {
        let infoval = match info.1 {
            None => String::from("unknown"),
            Some(ref val) => String::from(val),
        };
        msgs.push(ReportMessage{
            level: LogLevel::Info,
            content: format!("{} {}", info.0, infoval),
        });
    }
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("Uptime (s):               {}", sys.uptime()),
    });
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("Last boot (unix time):    {}", sys.boot_time()),
    });

    // average cpu load
    let cpu_load_avg = sys.load_average();
    let msg_level = match cpu_load_avg.five {
        pct if pct >= 80.0 => LogLevel::Error,
        pct if pct >= 50.0 => LogLevel::Warn,
        _ => LogLevel::Info,
    };
    msgs.push(ReportMessage{
        level: msg_level,
        content: format!(
            "Avg. CPU load: [1 min] {:.3}% | [5 min] {:.3}% | [15 min] {:.3}%",
            cpu_load_avg.one,
            cpu_load_avg.five,
            cpu_load_avg.fifteen,
        ),
    });

    // Then let's print the temperature of the different components:
    for component in sys.components() {
        let msg_content = format!("Temperaure of {:?}", &component);
        let critical_temp: f32 = 90.0;
        let msg_level = match component.temperature() {
            t if t >= critical_temp => LogLevel::Error,
            t if t >= 0.8 * component.max() => LogLevel::Warn,
            _ => LogLevel::Info,
        };
        msgs.push(ReportMessage{
            level: msg_level,
            content: msg_content,
        });
        msgs.push(match component.critical() {
            Some(ref critical_temp) => {
                ReportMessage{
                    level: LogLevel::Debug,
                    content: format!("Detected critical temp for {:}: {:}C", component.label(), critical_temp),
                }},
            None => {
                ReportMessage{
                    level: LogLevel::Debug,
                    content: format!("Estimated critical temp for {:}: {:}C", component.label(), critical_temp),
                }},
            });
    }

    // And finally the RAM and SWAP information:
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("total memory: {} kB", sys.total_memory()),
    });
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("used memory : {} kB", sys.used_memory()),
    });
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("total swap  : {} kB", sys.total_swap()),
    });
    msgs.push(ReportMessage{
        level: LogLevel::Info,
        content: format!("used swap   : {} kB", sys.used_swap()),
    });
    return msgs
}
