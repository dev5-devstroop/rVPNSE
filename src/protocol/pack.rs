//! SoftEther VPN PACK Binary Format Implementation
//!
//! This module implements the PACK binary format used by SoftEther VPN for
//! all data communication after the HTTP watermark handshake. PACK is SoftEther's
//! proprietary binary serialization format for key-value data structures.

use crate::error::{Result, VpnError};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// PACK element types (from SoftEther VPN source)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum ElementType {
    Int = 0,
    Data = 1,
    Str = 2,
    UniStr = 3,
    Int64 = 4,
}

impl TryFrom<u32> for ElementType {
    type Error = VpnError;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(ElementType::Int),
            1 => Ok(ElementType::Data),
            2 => Ok(ElementType::Str),
            3 => Ok(ElementType::UniStr),
            4 => Ok(ElementType::Int64),
            _ => Err(VpnError::Protocol(format!("Invalid element type: {}", value))),
        }
    }
}

/// PACK value variants
#[derive(Debug, Clone)]
pub enum Value {
    Int(u32),
    Int64(u64),
    Data(Vec<u8>),
    Str(String),
    UniStr(String), // UTF-16 string converted to UTF-8
}

impl Value {
    /// Get the element type for this value
    pub fn element_type(&self) -> ElementType {
        match self {
            Value::Int(_) => ElementType::Int,
            Value::Int64(_) => ElementType::Int64,
            Value::Data(_) => ElementType::Data,
            Value::Str(_) => ElementType::Str,
            Value::UniStr(_) => ElementType::UniStr,
        }
    }

    /// Serialize value to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Value::Int(i) => i.to_le_bytes().to_vec(),
            Value::Int64(i) => i.to_le_bytes().to_vec(),
            Value::Data(data) => data.clone(),
            Value::Str(s) => s.as_bytes().to_vec(),
            Value::UniStr(s) => {
                // Convert to UTF-16LE (as SoftEther expects)
                let utf16: Vec<u16> = s.encode_utf16().collect();
                let mut bytes = Vec::with_capacity(utf16.len() * 2);
                for code_unit in utf16 {
                    bytes.extend_from_slice(&code_unit.to_le_bytes());
                }
                bytes
            }
        }
    }

    /// Deserialize value from bytes
    pub fn from_bytes(element_type: ElementType, data: &[u8]) -> Result<Self> {
        match element_type {
            ElementType::Int => {
                if data.len() != 4 {
                    return Err(VpnError::Protocol("Invalid Int data length".to_string()));
                }
                let bytes: [u8; 4] = data.try_into().unwrap();
                Ok(Value::Int(u32::from_le_bytes(bytes)))
            }
            ElementType::Int64 => {
                if data.len() != 8 {
                    return Err(VpnError::Protocol("Invalid Int64 data length".to_string()));
                }
                let bytes: [u8; 8] = data.try_into().unwrap();
                Ok(Value::Int64(u64::from_le_bytes(bytes)))
            }
            ElementType::Data => Ok(Value::Data(data.to_vec())),
            ElementType::Str => {
                let s = String::from_utf8(data.to_vec())
                    .map_err(|_| VpnError::Protocol("Invalid UTF-8 string".to_string()))?;
                Ok(Value::Str(s))
            }
            ElementType::UniStr => {
                if data.len() % 2 != 0 {
                    return Err(VpnError::Protocol("Invalid UniStr data length".to_string()));
                }
                let mut utf16_codes = Vec::with_capacity(data.len() / 2);
                for chunk in data.chunks_exact(2) {
                    let code = u16::from_le_bytes([chunk[0], chunk[1]]);
                    utf16_codes.push(code);
                }
                let s = String::from_utf16(&utf16_codes)
                    .map_err(|_| VpnError::Protocol("Invalid UTF-16 string".to_string()))?;
                Ok(Value::UniStr(s))
            }
        }
    }
}

/// PACK element containing name and values
#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub values: Vec<Value>,
}

impl Element {
    /// Create a new element with a single value
    pub fn new(name: String, value: Value) -> Self {
        Self {
            name,
            values: vec![value],
        }
    }

    /// Create a new element with multiple values (array)
    pub fn new_array(name: String, values: Vec<Value>) -> Self {
        Self { name, values }
    }

    /// Get the element type (all values must be the same type)
    pub fn element_type(&self) -> Result<ElementType> {
        if self.values.is_empty() {
            return Err(VpnError::Protocol("Element has no values".to_string()));
        }
        
        let element_type = self.values[0].element_type();
        
        // Verify all values have the same type
        for value in &self.values {
            if value.element_type() != element_type {
                return Err(VpnError::Protocol(
                    "All values in element must have the same type".to_string()
                ));
            }
        }
        
        Ok(element_type)
    }
}

/// PACK structure containing elements
#[derive(Debug, Clone)]
pub struct Pack {
    pub elements: Vec<Element>,
}

impl Pack {
    /// Create a new empty PACK
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Add an element to the PACK
    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    /// Add an integer value
    pub fn add_int(&mut self, name: &str, value: u32) {
        self.elements.push(Element::new(name.to_string(), Value::Int(value)));
    }

    /// Add an integer array
    pub fn add_int_array(&mut self, name: &str, values: Vec<u32>) {
        let values: Vec<Value> = values.into_iter().map(Value::Int).collect();
        self.elements.push(Element::new_array(name.to_string(), values));
    }

    /// Add a 64-bit integer value
    pub fn add_int64(&mut self, name: &str, value: u64) {
        self.elements.push(Element::new(name.to_string(), Value::Int64(value)));
    }

    /// Add binary data
    pub fn add_data(&mut self, name: &str, data: Vec<u8>) {
        self.elements.push(Element::new(name.to_string(), Value::Data(data)));
    }

    /// Add a string value
    pub fn add_str(&mut self, name: &str, value: &str) {
        self.elements.push(Element::new(name.to_string(), Value::Str(value.to_string())));
    }

    /// Add a Unicode string value
    pub fn add_unistr(&mut self, name: &str, value: &str) {
        self.elements.push(Element::new(name.to_string(), Value::UniStr(value.to_string())));
    }

    /// Add an IP address (as integer)
    pub fn add_ip(&mut self, name: &str, ip: IpAddr) {
        match ip {
            IpAddr::V4(ipv4) => {
                let ip_int = u32::from(ipv4);
                self.add_int(name, ip_int);
            }
            IpAddr::V6(ipv6) => {
                // For IPv6, store as binary data
                self.add_data(name, ipv6.octets().to_vec());
            }
        }
    }

    /// Get an element by name
    pub fn get_element(&self, name: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.name == name)
    }

    /// Get an integer value
    pub fn get_int(&self, name: &str) -> Option<u32> {
        self.get_element(name)?
            .values.first()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            })
    }

    /// Get a 64-bit integer value
    pub fn get_int64(&self, name: &str) -> Option<u64> {
        self.get_element(name)?
            .values.first()
            .and_then(|v| match v {
                Value::Int64(i) => Some(*i),
                _ => None,
            })
    }

    /// Get binary data
    pub fn get_data(&self, name: &str) -> Option<&Vec<u8>> {
        self.get_element(name)?
            .values.first()
            .and_then(|v| match v {
                Value::Data(data) => Some(data),
                _ => None,
            })
    }

    /// Get a string value
    pub fn get_str(&self, name: &str) -> Option<&String> {
        self.get_element(name)?
            .values.first()
            .and_then(|v| match v {
                Value::Str(s) | Value::UniStr(s) => Some(s),
                _ => None,
            })
    }

    /// Serialize PACK to binary format (compatible with SoftEther)
    pub fn to_bytes(&self) -> Result<Bytes> {
        let mut buf = BytesMut::new();

        // Write number of elements (4 bytes, little-endian)
        buf.put_u32_le(self.elements.len() as u32);

        // Write each element
        for element in &self.elements {
            self.write_element(&mut buf, element)?;
        }

        Ok(buf.freeze())
    }

    /// Write a single element to the buffer
    fn write_element(&self, buf: &mut BytesMut, element: &Element) -> Result<()> {
        let element_type = element.element_type()?;

        // Write element name length and name (with null terminator)
        let name_bytes = element.name.as_bytes();
        buf.put_u32_le(name_bytes.len() as u32 + 1); // +1 for null terminator
        buf.put_slice(name_bytes);
        buf.put_u8(0); // null terminator

        // Write element type
        buf.put_u32_le(element_type as u32);

        // Write number of values
        buf.put_u32_le(element.values.len() as u32);

        // Write each value
        for value in &element.values {
            let value_bytes = value.to_bytes();
            buf.put_u32_le(value_bytes.len() as u32);
            buf.put_slice(&value_bytes);
        }

        Ok(())
    }

    /// Deserialize PACK from binary format
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        if data.len() < 4 {
            return Err(VpnError::Protocol("PACK data too short".to_string()));
        }

        // Read number of elements
        let num_elements = data.get_u32_le();
        let mut elements = Vec::with_capacity(num_elements as usize);

        // Read each element
        for _ in 0..num_elements {
            let element = Self::read_element(&mut data)?;
            elements.push(element);
        }

        Ok(Self { elements })
    }

    /// Read a single element from the buffer
    fn read_element(data: &mut Bytes) -> Result<Element> {
        if data.len() < 4 {
            return Err(VpnError::Protocol("Not enough data for element name length".to_string()));
        }

        // Read element name length
        let name_len = data.get_u32_le() as usize;
        if data.len() < name_len {
            return Err(VpnError::Protocol("Not enough data for element name".to_string()));
        }

        // Read element name (excluding null terminator)
        let name_bytes = data.copy_to_bytes(name_len);
        let name = String::from_utf8(name_bytes[..name_len.saturating_sub(1)].to_vec())
            .map_err(|_| VpnError::Protocol("Invalid element name UTF-8".to_string()))?;

        if data.len() < 8 {
            return Err(VpnError::Protocol("Not enough data for element type and value count".to_string()));
        }

        // Read element type
        let element_type = ElementType::try_from(data.get_u32_le())?;

        // Read number of values
        let num_values = data.get_u32_le() as usize;
        let mut values = Vec::with_capacity(num_values);

        // Read each value
        for _ in 0..num_values {
            if data.len() < 4 {
                return Err(VpnError::Protocol("Not enough data for value length".to_string()));
            }

            let value_len = data.get_u32_le() as usize;
            if data.len() < value_len {
                return Err(VpnError::Protocol("Not enough data for value".to_string()));
            }

            let value_bytes = data.copy_to_bytes(value_len);
            let value = Value::from_bytes(element_type, &value_bytes)?;
            values.push(value);
        }

        Ok(Element {
            name,
            values,
        })
    }
}

impl Default for Pack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_creation() {
        let mut pack = Pack::new();
        pack.add_int("test_int", 42);
        pack.add_str("test_str", "hello");
        pack.add_data("test_data", vec![1, 2, 3, 4]);

        assert_eq!(pack.get_int("test_int"), Some(42));
        assert_eq!(pack.get_str("test_str"), Some(&"hello".to_string()));
        assert_eq!(pack.get_data("test_data"), Some(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_pack_serialization() {
        let mut pack = Pack::new();
        pack.add_int("test_int", 42);
        pack.add_str("test_str", "hello");

        let bytes = pack.to_bytes().unwrap();
        let deserialized = Pack::from_bytes(bytes).unwrap();

        assert_eq!(deserialized.get_int("test_int"), Some(42));
        assert_eq!(deserialized.get_str("test_str"), Some(&"hello".to_string()));
    }

    #[test]
    fn test_ip_address_handling() {
        let mut pack = Pack::new();
        let ipv4 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        pack.add_ip("server_ip", ipv4);

        let serialized = pack.to_bytes().unwrap();
        let deserialized = Pack::from_bytes(serialized).unwrap();

        // IPv4 stored as integer
        assert!(deserialized.get_int("server_ip").is_some());
    }

    #[test]
    fn test_unicode_string() {
        let mut pack = Pack::new();
        pack.add_unistr("unicode_test", "Hello 世界 🌍");

        let serialized = pack.to_bytes().unwrap();
        let deserialized = Pack::from_bytes(serialized).unwrap();

        assert_eq!(deserialized.get_str("unicode_test"), Some(&"Hello 世界 🌍".to_string()));
    }
}
