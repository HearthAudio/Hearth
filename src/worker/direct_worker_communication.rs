use futures::executor;
use crate::utils::generic_connector::{DirectWorkerCommunication, DWCActionType};
use crate::worker::queue_processor::{LeaveAction, ProcessorIncomingAction, ProcessorIPC, ProcessorIPCData};

pub fn parse_dwc_message(dwc: DirectWorkerCommunication,ipc: &mut ProcessorIPC) {
    match dwc.action_type {
        DWCActionType::LeaveChannel => {
            ipc.sender.send(ProcessorIPCData {
                action: ProcessorIncomingAction::Actions(DWCActionType::LeaveChannel),
                job_id: dwc.job_id,
                songbird: None,
                leave_action: Some(LeaveAction {
                    guild_id: dwc.leave_channel_guild_id.unwrap().parse().unwrap()
                }),
            }).expect("Sending DWC Failed");
        }
        DWCActionType::PlayDirectLink => {
            ipc.sender.send(ProcessorIPCData {
                action: ProcessorIncomingAction::Actions(DWCActionType::PlayDirectLink),
                job_id: dwc.job_id,
                songbird: None,
                leave_action: None
            }).expect("Sending DWC Failed");
        }
        _ => {}
    }
}