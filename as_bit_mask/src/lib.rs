pub trait AsBitMask<const N: usize> {
    fn as_bytes(&self) -> [u8; N];
    fn from_bytes(bytes: &[u8; N]) -> Self;
}

pub use as_bit_mask_derive::AsBitMask;
pub use as_bit_mask_derive::AsBitMaskExplicit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte_test() {
        #[derive(as_bit_mask_derive::AsBitMask, Debug, PartialEq)]
        pub struct MotorConfigOptions{
            enabled: bool,
            limited_position: bool,
            stop_on_estop: bool,
        }
        
        let config = MotorConfigOptions::from_bytes(&[0b111]);
        let config_byte = MotorConfigOptions{
            enabled: true,
            limited_position: true,
            stop_on_estop: true,
        };
        assert_eq!(config, config_byte);
        for i in 0u8..8 {
            let config = MotorConfigOptions::from_bytes(&[i]);
            let raw_config = config.as_bytes()[0];
            assert_eq!(raw_config, i);       
        }
    }

    #[test]
    fn multi_byte_test() {
        #[derive(as_bit_mask_derive::AsBitMask, Debug, PartialEq)]
        pub struct MultiByteStruct{
            a: bool,
            b: bool,
            c: bool,
            d: bool,
            e: bool,
            f: bool,
            g: bool,
            h: bool,
            i: bool,
            j: bool,
            k: bool,
            l: bool,
        }

        for i in 0u16..(1<<12) {
            let config = MultiByteStruct::from_bytes(&u16::to_le_bytes(i));
            let raw_config = u16::from_le_bytes(config.as_bytes());
            assert_eq!(raw_config, i);
        }
    }

    #[test]
    fn multi_byte_test_explicit() {
        #[derive(as_bit_mask_derive::AsBitMaskExplicit, Debug, PartialEq)]
        pub struct MultiByteStruct{
            #[index(0)]
            a: bool,
            #[index(1)]
            b: bool,
            #[index(3)]
            c: bool,
            #[index(2)]
            d: bool,
            #[index(6)]
            e: bool,
            #[index(7)]
            f: bool,
            #[index(4)]
            g: bool,
            #[index(5)]
            h: bool,
            #[index(8)]
            i: bool,
            #[index(10)]
            j: bool,
            #[index(30)]
            k: bool,
            #[index(9)]
            l: bool,
            
        }

        for i in 0u32..(1<<16) {
            let v = u32::to_le_bytes(i);
            let config = MultiByteStruct::from_bytes(&v);
            let reconstructed = MultiByteStruct::from_bytes(&config.as_bytes());
            assert_eq!(config, reconstructed);
        }
    }
}
