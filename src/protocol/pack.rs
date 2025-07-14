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
            _ => {
                // Check if the value is too large to be a valid element type
                if value > 10000 {
                    return Err(VpnError::Protocol(format!("Element type {} is too large, likely corrupted data", value)));
                }
                
                // Log unknown element types but try to handle as Data for compatibility
                log::warn!("Unknown element type {}, treating as Data", value);
                Ok(ElementType::Data)
            }
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
            Value::Int(i) => i.to_be_bytes().to_vec(), // SoftEther uses big-endian
            Value::Int64(i) => i.to_be_bytes().to_vec(), // SoftEther uses big-endian
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
                // SoftEther uses big-endian for integers
                Ok(Value::Int(u32::from_be_bytes(bytes)))
            }
            ElementType::Int64 => {
                if data.len() != 8 {
                    return Err(VpnError::Protocol("Invalid Int64 data length".to_string()));
                }
                let bytes: [u8; 8] = data.try_into().unwrap();
                // SoftEther uses big-endian for integers
                Ok(Value::Int64(u64::from_be_bytes(bytes)))
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
                    // SoftEther uses little-endian for UTF-16 characters
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
    
    /// Get all data values from this element
    pub fn get_data_values(&self) -> Vec<&Vec<u8>> {
        self.values.iter().filter_map(|v| match v {
            Value::Data(data) => Some(data),
            _ => None,
        }).collect()
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

    /// Get all elements as a HashMap for easy iteration
    pub fn get_elements(&self) -> std::collections::HashMap<String, &Element> {
        self.elements.iter().map(|e| (e.name.clone(), e)).collect()
    }

    /// Serialize PACK to binary format (compatible with SoftEther)
    pub fn to_bytes(&self) -> Result<Bytes> {
        let mut buf = BytesMut::new();

        // Write number of elements (4 bytes, big-endian - SoftEther format)
        buf.put_u32(self.elements.len() as u32);

        // Write each element
        for element in &self.elements {
            self.write_element(&mut buf, element)?;
        }

        Ok(buf.freeze())
    }

    /// Write a single element to the buffer
    fn write_element(&self, buf: &mut BytesMut, element: &Element) -> Result<()> {
        let element_type = element.element_type()?;

        // Write element name length and name (with null terminator, big-endian)
        let name_bytes = element.name.as_bytes();
        buf.put_u32(name_bytes.len() as u32 + 1); // +1 for null terminator
        buf.put_slice(name_bytes);
        buf.put_u8(0); // null terminator

        // Write element type (big-endian)
        buf.put_u32(element_type as u32);

        // Write number of values (big-endian)
        buf.put_u32(element.values.len() as u32);

        // Write each value
        for value in &element.values {
            let value_bytes = value.to_bytes();
            buf.put_u32(value_bytes.len() as u32); // value length (big-endian)
            buf.put_slice(&value_bytes);
        }

        Ok(())
    }

    /// Deserialize PACK from binary format
    pub fn from_bytes(mut data: Bytes) -> Result<Self> {
        log::debug!("Parsing PACK from {} bytes", data.len());
        log::debug!("Raw bytes (first 64): {:?}", &data[..std::cmp::min(64, data.len())]);
        
        // Let's examine the entire PACK structure first
        if data.len() >= 16 {
            log::debug!("Detailed byte analysis:");
            log::debug!("Bytes 0-3 (num elements): {:?} = {}", &data[0..4], u32::from_be_bytes([data[0], data[1], data[2], data[3]]));
            log::debug!("Bytes 4-7 (first name len): {:?} = {}", &data[4..8], u32::from_be_bytes([data[4], data[5], data[6], data[7]]));
            if data.len() >= 32 {
                log::debug!("Bytes 8-31: {:?}", &data[8..32]);
            }
        }
        
        let original_len = data.len();
        
        if data.len() < 4 {
            return Err(VpnError::Protocol("PACK data too short".to_string()));
        }

        // Read number of elements (SoftEther uses big-endian)
        let num_elements = data.get_u32();
        log::debug!("PACK contains {} elements (big-endian), consumed 4 bytes, {} remaining", num_elements, data.len());
        
        // Sanity check: element count shouldn't be too large
        if num_elements > 10000 {
            return Err(VpnError::Protocol(format!("Element count {} seems too large", num_elements)));
        }
        
        let mut elements = Vec::with_capacity(num_elements as usize);

        // Read each element with graceful error handling
        for i in 0..num_elements {
            let bytes_before = data.len();
            log::debug!("Parsing element {} of {}, bytes remaining before element: {}, offset: {}", 
                       i + 1, num_elements, data.len(), bytes_before - data.len());
        
            // Add detailed hex dump of the next 16 bytes for debugging
            if data.len() >= 16 {
                let debug_bytes = &data[..16];
                log::debug!("Next 16 bytes at element start: {:02x?}", debug_bytes);
            }
            
            // Try to parse element, but be tolerant of failures after the first element
            match Self::read_element(&mut data) {
                Ok(element) => {
                    let bytes_after = data.len();
                    log::debug!("Parsed element: name={}, values={}, consumed {} bytes", 
                               element.name, element.values.len(), bytes_before - bytes_after);
                    elements.push(element);

                    // After parsing first element, show what's next for debugging
                    if i == 0 && data.len() >= 16 {
                        log::debug!("After first element: {} bytes remaining", data.len());
                        log::debug!("Next 16 bytes after first element: {:02x?}", &data[..16]);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse element {} of {}: {}", i + 1, num_elements, e);
                    
                    // If we successfully parsed at least one element (especially the first one with error info),
                    // we can continue with what we have
                    if i > 0 {
                        log::info!("Successfully parsed {} of {} elements, continuing with partial PACK", i, num_elements);
                        break;
                    } else {
                        // If we can't parse the first element, that's a real problem
                        return Err(e);
                    }
                }
            }
        }

        log::debug!("Successfully parsed PACK with {} elements", elements.len());
        Ok(Self { elements })
    }

    /// Read a single element from the buffer
    fn read_element(data: &mut Bytes) -> Result<Element> {
        let bytes_before = data.len();
        let original_len = bytes_before; // For offset calculation
        
        if data.len() < 4 {
            return Err(VpnError::Protocol("Not enough data for element name length".to_string()));
        }

        // Read element name length (big-endian, includes null terminator)
        let name_len_raw = data.get_u32();
        log::debug!("Element name length raw: {} (includes null terminator), consumed 4 bytes, {} remaining", name_len_raw, data.len());
        
        // Safety check: reject unreasonably large name lengths
        if name_len_raw > 1000 { // 1KB limit for element names
            return Err(VpnError::Protocol(format!("Element name length {} is unreasonably large", name_len_raw)));
        }
        
        let name_len = name_len_raw as usize;
        
        if name_len == 0 {
            return Err(VpnError::Protocol("Element name length is zero".to_string()));
        }
        
        if data.len() < name_len {
            return Err(VpnError::Protocol("Not enough data for element name".to_string()));
        }

        // Read element name (SoftEther format: length includes +1 for null, but data doesn't include null)
        let name_bytes = data.copy_to_bytes(name_len);
        // SoftEther string format: length includes +1 for null terminator, but actual data is just the string
        let actual_name_len = name_len.saturating_sub(1);
        let name = String::from_utf8(name_bytes[..actual_name_len].to_vec())
            .map_err(|_| VpnError::Protocol("Invalid element name UTF-8".to_string()))?;
        log::debug!("Element name: '{}', consumed {} bytes, {} remaining", name, name_len, data.len());
        
        // SoftEther PACK format: element name data is padded to 4-byte boundary
        // We need to pad just the name data (not including the length field)
        let padded_name_len = (name_len + 3) & !3; // Round name_len up to 4-byte boundary
        let padding_needed = padded_name_len - name_len;
        
        if padding_needed > 0 && data.len() >= padding_needed {
            let padding = data.copy_to_bytes(padding_needed);
            log::debug!("Skipped {} name padding bytes: {:?}, {} remaining", padding_needed, padding, data.len());
        }
        
        // Additional alignment: SoftEther appears to need one more byte alignment after string padding
        // Based on the binary analysis, there's an extra 0x00 byte that we need to skip
        if data.len() > 0 && data[0] == 0 {
            let extra_byte = data.get_u8();
            log::debug!("Skipped extra alignment byte: 0x{:02x}, {} remaining", extra_byte, data.len());
        }
        
        log::debug!("After name + padding, next 12 bytes: {:?}", &data[..std::cmp::min(12, data.len())]);

        if data.len() < 8 {
            return Err(VpnError::Protocol("Not enough data for element type and value count".to_string()));
        }

        // Read element type (big-endian)
        log::debug!("About to read element type, next 12 bytes: {:?}", 
                   &data[..std::cmp::min(12, data.len())]);
        log::debug!("Element type raw bytes (hex): {:02x} {:02x} {:02x} {:02x}", 
                   data[0], data[1], data[2], data[3]);
        let element_type_raw = data.get_u32();
        log::debug!("Element type raw: {} (0x{:08x}), consumed 4 bytes, {} remaining", 
                   element_type_raw, element_type_raw, data.len());
        
        // Convert to decimal to see what we're getting
        log::debug!("Expected element type range: 0-4, got: {}", element_type_raw);
        
        let element_type = ElementType::try_from(element_type_raw)?;
        log::debug!("Element type: {:?}", element_type);

        // Read number of values (big-endian)
        log::debug!("About to read num values, next 8 bytes: {:?}", 
                   &data[..std::cmp::min(8, data.len())]);
        let num_values_raw = data.get_u32();
        log::debug!("Number of values raw: {}, consumed 4 bytes, {} remaining", num_values_raw, data.len());
        let num_values = num_values_raw as usize;
        log::debug!("Number of values: {}", num_values);
        
        let mut values = Vec::with_capacity(num_values);

        // Read each value
        for j in 0..num_values {
            if data.len() < 4 {
                return Err(VpnError::Protocol("Not enough data for value length".to_string()));
            }

            let value_len_raw = data.get_u32();
            log::debug!("Value {} length raw: {}, consumed 4 bytes, {} remaining", j, value_len_raw, data.len());
            
            // Safety check: reject unreasonably large values to prevent memory allocation attacks
            if value_len_raw > 10_000_000 { // 10MB limit per value
                log::error!("Value {} length {} is unreasonably large, likely corrupted data", j, value_len_raw);
                return Err(VpnError::Protocol(format!("Value length {} exceeds safety limit", value_len_raw)));
            }
            
            let value_len = value_len_raw as usize;
            log::debug!("Value {} length: {}", j, value_len);
            
            if data.len() < value_len {
                log::error!("Value {} claims length {} but only {} bytes remaining. Raw bytes around position: {:?}", 
                           j, value_len, data.len(), &data[..std::cmp::min(16, data.len())]);
                return Err(VpnError::Protocol(format!("Not enough data for value {} (need {}, have {})", j, value_len, data.len())));
            }

            let value_bytes = data.copy_to_bytes(value_len);
            log::debug!("Value {} bytes: {:?}", j, &value_bytes[..std::cmp::min(8, value_bytes.len())]);
            let value = Value::from_bytes(element_type, &value_bytes)?;
            log::debug!("Value {}: {:?}, consumed {} bytes, {} remaining", j, value, value_len, data.len());
            values.push(value);
            
            // SoftEther PACK format: values are padded to 4-byte boundary
            let padded_value_len = (value_len + 3) & !3; // Round up to 4-byte boundary
            let value_padding_needed = padded_value_len - value_len;
            
            if value_padding_needed > 0 && data.len() >= value_padding_needed {
                let value_padding = data.copy_to_bytes(value_padding_needed);
                log::debug!("Skipped {} value padding bytes: {:?}, {} remaining", value_padding_needed, value_padding, data.len());
            }
        }

        let bytes_after = data.len();
        log::debug!("Element '{}' parsing complete, total consumed: {} bytes", name, bytes_before - bytes_after);

        // SoftEther PACK format: Try exactly 3 bytes of inter-element padding
        // This should get us from [00, 00, 01, 00] to [00, 00, 00, ??] for the next name length
        if data.len() >= 3 && data[0] == 0x00 && data[1] == 0x00 {
            let padding1 = data.get_u8();
            let padding2 = data.get_u8();
            let padding3 = data.get_u8();
            log::debug!("Applied 3 bytes inter-element padding: 0x{:02x} 0x{:02x} 0x{:02x}, {} remaining", 
                       padding1, padding2, padding3, data.len());
        }

        let total_element_size = bytes_before - data.len();
        log::debug!("Total element size with padding: {}, consumed {} bytes", total_element_size, bytes_before - data.len());

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
