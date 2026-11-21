// Full packets recorded from the unchanged repository encoder with an injected
// 16-byte RNG buffer. The first three tails are its existing captured vectors;
// all other credentials are synthetic boundary cases, not third-party vectors.
pub struct Case<'a> {
    pub name: &'static str,
    pub username: &'a str,
    pub password: &'a str,
    pub locale: &'a str,
    pub timestamp: u32,
    pub packet: &'static str,
}

pub fn cases(password: &str) -> Vec<Case<'_>> {
    [
        ("captured1", "admin", password, "DEU", 0x34fde6a4, "00010234a405060708e60a0b0c0dfd0f010000002000000000000000efbeedfeec22eb7bcfc676f229420875226c1a5c4db9a5b92e84509093b3d9280d314a83"),
        ("captured2", "admin", password, "DEU", 0x351474df, "11121335df16171819741b1c1d1e1420010000002000000000000000efbeedfe27d39c95ecba503f2ba34c6cd8735af5bebf9e06529d2c729139f6bcab6e4955"),
        ("captured3", "admin", password, "DEU", 0x35149faf, "22232435af2728292a9f2c2d2e2f1431010000002000000000000000efbeedfe1cb00679ce23a0d5d6c1fbe58544c0d57d48767c2f161baebaabee8c397920d1"),
        ("empty_zero", "", "", "", 0, "333435000038393a3b003d3e3f400042010000002000000000000000efbeedfe632598a34ab0f018c8a095bf902baa86632598a34ab0f018c8a095bf902baa86"),
        ("unicode_max", "用戶🙂", "päss🔑", "日本語", u32::MAX, "444546ffff494a4b4cff4e4f5051ff53010000002000000000000000efbeedfe1adee59396b20dc23041b9013d98475e471a2edb5ea734a4e3cfa07d481250e6"),
        ("embedded_nul", "a\0b", "\0p\0", "E\0N", 1, "55565700015a5b5c5d005f6061620064010000002000000000000000efbeedfe35445666a7cdcd2e7690b3cf3368bc77625aede2c8d57facb589e5e1480249c6"),
        ("username31", "1234567890123456789012345678901", "omitted", "ENU", 0x12345678, "66676812786b6c6d6e56707172733475010000002000000000000000efbeedfe0518ffdcccf6133a64914880d373e90973d44d61fe49c789e21291a511412690"),
        ("username32", "12345678901234567890123456789012", "omitted", "ENU", 0x80000000, "77787980007c7d7e7f00818283840086010000002000000000000000efbeedfe33fcbeafb1455ec42d342f64b6ec2c03c91aa98228841866ddf1d083da0646b0"),
        ("username33", "123456789012345678901234567890123", "omitted", "ENU", 0x80000000, "88898a80008d8e8f9000929394950097010000002000000000000000efbeedfe33fcbeafb1455ec42d342f64b6ec2c03c91aa98228841866ddf1d083da0646b0"),
        ("split_username", "1234567890123456789012345678901é", "omitted", "ENU", 0x01020304, "999a9b01049e9fa0a103a3a4a5a602a8010000002000000000000000efbeedfefb705a9d57bbf0145570bf086abdcfcc046bfab04f3a018149d71ebd5728b2d8"),
        ("split_password", "user", "12345678901234567890123456🙂", "ENU", 0x00001000, "aaabac0000afb0b1b210b4b5b6b700b9010000002000000000000000efbeedfebed9188bfadfaee84e3974b8421a7c5d0200b9154fe16548be01cb9b64805dac"),
        ("split_locale", "user", "123456789012345678901234", "日本語", 0x00010000, "bbbcbd0000c0c1c2c300c5c6c7c801ca010000002000000000000000efbeedfeb66a0f128c9eaabaefa09db19bdabf3bbe71e8987172764afce03705296495f2"),
    ]
    .into_iter()
    .map(|(name, username, password, locale, timestamp, packet)| Case {
        name, username, password, locale, timestamp, packet,
    })
    .collect()
}

pub fn randomness(index: usize) -> [u8; 16] {
    std::array::from_fn(|i| (index as u8).wrapping_mul(17).wrapping_add(i as u8))
}

pub fn decode_packet(encoded: &str) -> [u8; 64] {
    assert_eq!(encoded.len(), 128);
    std::array::from_fn(|i| u8::from_str_radix(&encoded[i * 2..i * 2 + 2], 16).unwrap())
}
