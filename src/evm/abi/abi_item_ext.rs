use alloy::json_abi::{AbiItem,EventParam,Param};

pub trait AbiItemExt {
    fn scrub_param_names(&mut self);
}

impl AbiItemExt for AbiItem<'_> {
    fn scrub_param_names(&mut self) {
        match self {
            alloy::json_abi::AbiItem::Event(e) => e.to_mut().inputs.iter_mut().for_each(scrub_event_param_name),
            alloy::json_abi::AbiItem::Function(e) => {
                e.to_mut().inputs.iter_mut().for_each(scrub_param_name);
                e.to_mut().outputs.iter_mut().for_each(scrub_param_name);
            }
            _ => ()
        }
    }
}

fn scrub_event_param_name(p: &mut EventParam) {
    p.name = "".to_string();
    p.components.iter_mut().for_each(scrub_param_name);
}

fn scrub_param_name(p: &mut Param) {
    p.name = "".to_string();
    p.components.iter_mut().for_each(scrub_param_name);
}