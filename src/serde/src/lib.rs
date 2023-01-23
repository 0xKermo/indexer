//! Utilities used for serializing/deserializing sequencer REST API related data.

use ethers::types::{H128, H160, H256};
use num_bigint::BigUint;
use pathfinder_common::{
    CallParam, ConstructorParam, EthereumAddress, EventData, EventKey, Fee, GasPrice,
    L1ToL2MessagePayloadElem, L2ToL1MessagePayloadElem, StarknetBlockNumber,
    TransactionSignatureElem, TransactionVersion,
};
use serde::de::Visitor;
use serde_with::{serde_conv, DeserializeAs, SerializeAs};
use stark_hash::{Felt, HexParseError, OverflowError};
use std::borrow::Cow;
use std::str::FromStr;

serde_conv!(
    pub CallParamAsDecimalStr,
    CallParam,
    |serialize_me: &CallParam| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(CallParam)
);

serde_conv!(
    pub ConstructorParamAsDecimalStr,
    ConstructorParam,
    |serialize_me: &ConstructorParam| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(ConstructorParam)
);

serde_conv!(
    pub EventDataAsDecimalStr,
    EventData,
    |serialize_me: &EventData| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(EventData)
);

serde_conv!(
    pub EventKeyAsDecimalStr,
    EventKey,
    |serialize_me: &EventKey| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(EventKey)
);

serde_conv!(
    pub L1ToL2MessagePayloadElemAsDecimalStr,
    L1ToL2MessagePayloadElem,
    |serialize_me: &L1ToL2MessagePayloadElem| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(L1ToL2MessagePayloadElem)
);

serde_conv!(
    pub L2ToL1MessagePayloadElemAsDecimalStr,
    L2ToL1MessagePayloadElem,
    |serialize_me: &L2ToL1MessagePayloadElem| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(L2ToL1MessagePayloadElem)
);

serde_conv!(
    pub TransactionSignatureElemAsDecimalStr,
    TransactionSignatureElem,
    |serialize_me: &TransactionSignatureElem| starkhash_to_dec_str(&serialize_me.0),
    |s: &str| starkhash_from_dec_str(s).map(TransactionSignatureElem)
);

pub struct EthereumAddressAsHexStr;

impl SerializeAs<EthereumAddress> for EthereumAddressAsHexStr {
    fn serialize_as<S>(source: &EthereumAddress, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // EthereumAddress is "0x" + 40 digits at most
        let mut buf = [0u8; 2 + 40];
        let s = bytes_as_hex_str(source.0.as_bytes(), &mut buf);
        serializer.serialize_str(s)
    }
}

impl<'de> DeserializeAs<'de, EthereumAddress> for EthereumAddressAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<EthereumAddress, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EthereumAddressVisitor;

        impl<'de> Visitor<'de> for EthereumAddressVisitor {
            type Value = EthereumAddress;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 40 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<{ H160::len_bytes() }>(v)
                    .map_err(serde::de::Error::custom)
                    .map(|b| EthereumAddress(H160::from(b)))
            }
        }

        deserializer.deserialize_str(EthereumAddressVisitor)
    }
}

pub struct H256AsNoLeadingZerosHexStr;

impl SerializeAs<H256> for H256AsNoLeadingZerosHexStr {
    fn serialize_as<S>(source: &H256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // H256 is "0x" + 64 digits at most
        let mut buf = [0u8; 2 + 64];
        let s = bytes_as_hex_str(source.as_bytes(), &mut buf);
        serializer.serialize_str(s)
    }
}

impl<'de> DeserializeAs<'de, H256> for H256AsNoLeadingZerosHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<H256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct H256Visitor;

        impl<'de> Visitor<'de> for H256Visitor {
            type Value = H256;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 64 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<{ H256::len_bytes() }>(v)
                    .map_err(serde::de::Error::custom)
                    .map(H256::from)
            }
        }

        deserializer.deserialize_str(H256Visitor)
    }
}

pub struct FeeAsHexStr;

impl SerializeAs<Fee> for FeeAsHexStr {
    fn serialize_as<S>(source: &Fee, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Fee is "0x" + 32 digits at most
        let mut buf = [0u8; 2 + 32];
        let s = bytes_as_hex_str(source.0.as_bytes(), &mut buf);
        serializer.serialize_str(s)
    }
}

impl<'de> DeserializeAs<'de, Fee> for FeeAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<Fee, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FeeVisitor;

        impl<'de> Visitor<'de> for FeeVisitor {
            type Value = Fee;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 32 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<{ H128::len_bytes() }>(v)
                    .map_err(serde::de::Error::custom)
                    .map(|b| Fee(H128::from(b)))
            }
        }

        deserializer.deserialize_str(FeeVisitor)
    }
}

pub struct GasPriceAsHexStr;

impl SerializeAs<GasPrice> for GasPriceAsHexStr {
    fn serialize_as<S>(source: &GasPrice, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // GasPrice is "0x" + 32 digits at most
        let mut buf = [0u8; 2 + 32];
        let bytes = source.0.to_be_bytes();
        let s = bytes_as_hex_str(&bytes, &mut buf);
        serializer.serialize_str(s)
    }
}

impl<'de> DeserializeAs<'de, GasPrice> for GasPriceAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<GasPrice, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GasPriceVisitor;

        impl<'de> Visitor<'de> for GasPriceVisitor {
            type Value = GasPrice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 32 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                bytes_from_hex_str::<16>(v)
                    .map_err(serde::de::Error::custom)
                    .map(GasPrice::from_be_bytes)
            }
        }

        deserializer.deserialize_str(GasPriceVisitor)
    }
}

pub struct StarknetBlockNumberAsHexStr;

impl SerializeAs<StarknetBlockNumber> for StarknetBlockNumberAsHexStr {
    fn serialize_as<S>(source: &StarknetBlockNumber, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = source.get().to_be_bytes();
        // StarknetBlockNumber is "0x" + 16 digits at most
        let mut buf = [0u8; 2 + 16];
        let s = bytes_as_hex_str(&bytes, &mut buf);
        serializer.serialize_str(s)
    }
}

impl<'de> DeserializeAs<'de, StarknetBlockNumber> for StarknetBlockNumberAsHexStr {
    fn deserialize_as<D>(deserializer: D) -> Result<StarknetBlockNumber, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StarknetBlockNumberVisitor;

        impl<'de> Visitor<'de> for StarknetBlockNumberVisitor {
            type Value = StarknetBlockNumber;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a hex string of up to 16 digits with an optional '0x' prefix")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let stripped = v.strip_prefix("0x").unwrap_or(v);
                let raw = u64::from_str_radix(stripped, 16).map_err(serde::de::Error::custom)?;
                StarknetBlockNumber::deserialize_value::<E>(raw)
            }
        }

        deserializer.deserialize_str(StarknetBlockNumberVisitor)
    }
}

serde_with::serde_conv!(
    pub TransactionVersionAsHexStr,
    TransactionVersion,
    |serialize_me: &TransactionVersion| bytes_to_hex_str(serialize_me.0.as_bytes()),
    |s: &str| bytes_from_hex_str::<{ H256::len_bytes() }>(s).map(|b| TransactionVersion(H256::from(b)))
);

serde_with::serde_conv!(
    pub H256AsHexStr,
    ethers::types::H256,
    |u: &H256| bytes_to_hex_str(u.as_bytes()),
    |s: &str| bytes_from_hex_str::<32>(s).map(H256::from)
);

serde_with::serde_conv!(
    pub U64AsHexStr,
    u64,
    |u: &u64| bytes_to_hex_str(&u.to_be_bytes()),
    |s: String| bytes_from_hex_str::<8>(&s).map(u64::from_be_bytes)
);

/// A helper conversion function. Only use with __sequencer API related types__.
fn starkhash_from_biguint(b: BigUint) -> Result<Felt, OverflowError> {
    Felt::from_be_slice(&b.to_bytes_be())
}

/// A helper conversion function. Only use with __sequencer API related types__.
pub fn starkhash_to_dec_str(h: &Felt) -> String {
    let b = h.to_be_bytes();
    let b = BigUint::from_bytes_be(&b);
    b.to_str_radix(10)
}

/// A helper conversion function. Only use with __sequencer API related types__.
fn starkhash_from_dec_str(s: &str) -> Result<Felt, anyhow::Error> {
    // TODO remove fallback to hex string representation once mainnet moves to cairo-0.8.0
    match BigUint::from_str(s) {
        Ok(b) => {
            let h = starkhash_from_biguint(b)?;
            Ok(h)
        }
        Err(_) => {
            let h = Felt::from_hex_str(s)?;
            Ok(h)
        }
    }
}

/// A convenience function which parses a hex string into a byte array.
///
/// Supports both upper and lower case hex strings, as well as an
/// optional "0x" prefix.
fn bytes_from_hex_str<const N: usize>(hex_str: &str) -> Result<[u8; N], HexParseError> {
    fn parse_hex_digit(digit: u8) -> Result<u8, HexParseError> {
        match digit {
            b'0'..=b'9' => Ok(digit - b'0'),
            b'A'..=b'F' => Ok(digit - b'A' + 10),
            b'a'..=b'f' => Ok(digit - b'a' + 10),
            other => Err(HexParseError::InvalidNibble(other)),
        }
    }

    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    if hex_str.len() > N * 2 {
        return Err(HexParseError::InvalidLength {
            max: N * 2,
            actual: hex_str.len(),
        });
    }

    let mut buf = [0u8; N];

    // We want the result in big-endian so reverse iterate over each pair of nibbles.
    let chunks = hex_str.as_bytes().rchunks_exact(2);

    // Handle a possible odd nibble remaining nibble.
    let odd_nibble = chunks.remainder();
    if !odd_nibble.is_empty() {
        let full_bytes = hex_str.len() / 2;
        buf[N - 1 - full_bytes] = parse_hex_digit(odd_nibble[0])?;
    }

    for (i, c) in chunks.enumerate() {
        // Indexing c[0] and c[1] are safe since chunk-size is 2.
        buf[N - 1 - i] = parse_hex_digit(c[0])? << 4 | parse_hex_digit(c[1])?;
    }

    Ok(buf)
}

/// The first stage of conversion - skip leading zeros
fn skip_zeros(bytes: &[u8]) -> (impl Iterator<Item = &u8>, usize, usize) {
    // Skip all leading zero bytes
    let it = bytes.iter().skip_while(|&&b| b == 0);
    let num_bytes = it.clone().count();
    let skipped = bytes.len() - num_bytes;
    // The first high nibble can be 0
    let start = if bytes[skipped] < 0x10 { 1 } else { 2 };
    // Number of characters to display
    let len = start + num_bytes * 2;
    (it, start, len)
}

/// The second stage of conversion - map bytes to hex str
fn it_to_hex_str<'a>(
    it: impl Iterator<Item = &'a u8>,
    start: usize,
    len: usize,
    buf: &'a mut [u8],
) -> &'a [u8] {
    const LUT: [u8; 16] = *b"0123456789abcdef";
    buf[0] = b'0';
    // Same small lookup table is ~25% faster than hex::encode_from_slice 🤷
    it.enumerate().for_each(|(i, &b)| {
        let idx = b as usize;
        let pos = start + i * 2;
        let x = [LUT[(idx & 0xf0) >> 4], LUT[idx & 0x0f]];
        buf[pos..pos + 2].copy_from_slice(&x);
    });
    buf[1] = b'x';
    &buf[..len]
}

/// A convenience function which produces a "0x" prefixed hex str slice in a given buffer `buf`
/// from an array of bytes.
/// Panics if `bytes.len() * 2 + 2 > buf.len()`
pub fn bytes_as_hex_str<'a>(bytes: &'a [u8], buf: &'a mut [u8]) -> &'a str {
    let expected_buf_len = bytes.len() * 2 + 2;
    assert!(
        buf.len() >= expected_buf_len,
        "buffer size is {}, expected at least {}",
        buf.len(),
        expected_buf_len
    );

    if !bytes.iter().any(|b| *b != 0) {
        return "0x0";
    }

    let (it, start, len) = skip_zeros(bytes);
    let res = it_to_hex_str(it, start, len, buf);
    // Unwrap is safe because `buf` holds valid UTF8 characters.
    std::str::from_utf8(res).unwrap()
}

/// A convenience function which produces a "0x" prefixed hex string from a [Felt].
#[allow(dead_code)]
pub fn bytes_to_hex_str(bytes: &[u8]) -> Cow<'static, str> {
    if !bytes.iter().any(|b| *b != 0) {
        return Cow::from("0x0");
    }
    let (it, start, len) = skip_zeros(bytes);
    let mut buf = vec![0u8; len];
    it_to_hex_str(it, start, len, &mut buf);
    // Unwrap is safe as the buffer contains valid utf8
    String::from_utf8(buf).unwrap().into()
}

/// Extract JSON representation of program and entry points from the contract definition.
pub fn extract_program_and_entry_points_by_type(
    contract_definition_dump: &[u8],
) -> anyhow::Result<(serde_json::Value, serde_json::Value)> {
    use anyhow::Context;

    #[derive(serde::Deserialize)]
    struct ContractDefinition {
        pub program: serde_json::Value,
        pub entry_points_by_type: serde_json::Value,
    }

    let contract_definition =
        serde_json::from_slice::<ContractDefinition>(contract_definition_dump)
            .context("Failed to parse contract_definition")?;
    Ok((
        contract_definition.program,
        contract_definition.entry_points_by_type,
    ))
}