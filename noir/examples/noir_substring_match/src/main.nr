use dep::noir_string_search::{StringBody, StringBody512, SubString, SubString32};

pub fn main(haystack_text: str<6>, needle_text: str<3>) -> pub bool {
    let haystack_text_bytes = haystack_text.as_bytes();
    let needle_text_bytes = needle_text.as_bytes();
    let mut haystack: StringBody512 = StringBody::new(haystack_text_bytes, haystack_text_bytes.len());
    let mut needle: SubString32 = SubString::new(needle_text_bytes, needle_text_bytes.len());
    let result = haystack.substring_match(needle);
    result.0
}