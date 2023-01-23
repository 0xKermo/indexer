use anyhow::Context;
use serde::{Deserialize, Serialize};

impl ContractClass {
    pub fn from_definition_bytes(data: &[u8]) -> anyhow::Result<ContractClass> {
        let mut json = serde_json::from_slice::<serde_json::Value>(data).context("Parsing json")?;
        let json_obj = json
            .as_object_mut()
            .context("Class definition is not a json object")?;
        // ABI is optional.
        let abi = json_obj.get_mut("abi").and_then(|json| {
            let json = json.take();
            serde_json::from_value::<Vec<ContractAbiEntry>>(json).ok()
        });
        
     

        Ok(ContractClass {
            abi,
        })
    }
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContractClass {
    pub abi: Option<Vec<ContractAbiEntry>>,
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum ContractAbiEntry {
    Function(FunctionAbiEntry),
    Event(EventAbiEntry),
    Struct(StructAbiEntry),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum StructAbiType {
    Struct,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum EventAbiType {
    Event,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum FunctionAbiType {
    Function,
    L1Handler,
    // This is missing from the v0.2 RPC specification and will be added in the
    // next version. We add it as a deviation from the current spec, since it is
    // effectively a bug in the v0.2 specification.
    Constructor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StructAbiEntry {
    pub r#type: StructAbiType,
    pub name: String,
    pub size: u64,
    pub members: Vec<StructMember>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StructMember {
    // Serde does not support deny_unknown_fields + flatten, so we
    // flatten TypedParameter manually here.
    #[serde(rename = "name")]
    typed_parameter_name: String,
    #[serde(rename = "type")]
    typed_parameter_type: String,
    offset: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EventAbiEntry {
    r#type: EventAbiType,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub keys: Option<Vec<TypedParameter>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub data: Option<Vec<TypedParameter>>,
    // The `inputs` and `outputs` property is not part of the JSON-RPC
    // specification, but because we use these types to parse the
    // `starknet_estimateFee` request and then serialize the class definition in
    // the transaction for the Python layer we have to keep this property when
    // serializing.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub inputs: Option<Vec<TypedParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub outputs: Option<Vec<TypedParameter>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FunctionAbiEntry {
    r#type: FunctionAbiType,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    inputs: Option<Vec<TypedParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    outputs: Option<Vec<TypedParameter>>,
    // This is not part of the JSON-RPC specification, but because we use these
    // types to parse the `starknet_estimateFee` request and then serialize the
    // class definition in the transaction for the Python layer we have to keep
    // this property when serializing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stateMutability")]
    state_mutability: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TypedParameter {
    pub name: String,
    r#type: String,
}
