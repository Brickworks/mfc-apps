use log::{trace, debug, info, warn, error};
use sysinfo::{ComponentExt, NetworkExt, NetworksExt, ProcessExt, System, SystemExt};

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
                debug!("{:#?}", &msg.content)
            }
            LogLevel::Info => {
                info!("{:#?}", &msg.content)
            },
            LogLevel::Warn => {
                warn!("{:#?}", &msg.content)
            },
            _ => {
                error!("{:#?}", &msg.content)
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

    // // Now let's print every process' id and name:
    // for (pid, proc_) in system.get_process_list() {
    //     println!("{}:{} => status: {:?}", pid, proc_.name, proc_.status);
    // }
    
    // Then let's print the temperature of the different components:
    for component in sys.components() {
        let msg_content = format!("Temperaure of {:?}", &component);
        let critical_temp = component.critical().unwrap_or(1.5*component.max());
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
                    content: format!("Detected critical temp for {:}: {:?}", component.label(), critical_temp),
                }},
            None => {
                ReportMessage{
                    level: LogLevel::Debug,
                    content: format!("Estimated critical temp for {:}: {:?}", component.label(), 1.5*component.max()),
                }},
            });
    }
    
    // And then all disks' information:
    for disk in sys.disks() {
        msgs.push(ReportMessage{
            level: LogLevel::Info,
            content: format!("{:?}", disk),
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
