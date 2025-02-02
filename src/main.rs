use chrono::DateTime;
use serde_derive::Deserialize;
use serde_json::value::RawValue;
use std::io;

#[derive(Deserialize)]
struct NotesV2 {
	notes_v2: Vec<Note>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Note {
	title: Box<RawValue>,
	text: Box<RawValue>,
	created_timestamp: i64,
	updated_timestamp: i64,
	tags: Vec<Tag>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Tag {
	name: String,
}

fn main() {
	let notes: NotesV2 = serde_json::from_reader(io::stdin()).unwrap();

	for note in notes.notes_v2 {
		let created_timestamp =
			DateTime::from_timestamp_millis(note.created_timestamp * 1000).unwrap();
		let updated_timestamp =
			DateTime::from_timestamp_millis(note.updated_timestamp * 1000).unwrap();
		let title_utf8 = from_utf16(&*note.title);
		let text_utf8 = from_utf16(&*note.text);
		println!(
			"[{}]-[{}]\n{}\n\n{}\n\n\n",
			created_timestamp, updated_timestamp, title_utf8, text_utf8
		);
	}
}

fn hex_digit(in_c: char) -> Option<u32> {
	let c = in_c.to_ascii_lowercase();
	if c >= '0' && c <= '9' {
		Some(c as u32 - '0' as u32)
	} else if c >= 'a' && c <= 'f' {
		Some(c as u32 - 'a' as u32 + 10)
	} else {
		None
	}
}

fn from_utf16(p0: &RawValue) -> String {
	enum State {
		Begin,
		End,
		Normal,
		Escaping,
		Unicode(i8, u32),
		Error,
	}
	let mut buf = Vec::new();
	let mut quoting = false;
	let mut state: State = State::Begin;
	for c in p0.get().chars() {
		match state {
			State::Begin => {
				quoting = c == '\"';
				state = State::Normal;
			}
			State::Normal => match c {
				'\\' => state = State::Escaping,
				'\"' => {
					if quoting {
						state = State::End;
					}
				}
				_ => buf.push(c as u8),
			},
			State::Error => {
				break;
			}
			State::End => {
				break;
			}
			State::Escaping => match c {
				'u' => state = State::Unicode(0, 0),
				'n' => {
					buf.push('\n' as u8);
					state = State::Normal;
				}
				't' => {
					buf.push('\t' as u8);
					state = State::Normal;
				}
				'r' => {
					buf.push('\r' as u8);
					state = State::Normal;
				}
				_ => {
					buf.push(c as u8);
					state = State::Normal;
				}
			},
			State::Unicode(n, v) => {
				if let Some(digit) = hex_digit(c) {
					let char_v = (v << 4) + digit;
					state = if n == 3 {
						buf.push(char::from_u32(char_v).unwrap() as u8);
						State::Normal
					} else {
						State::Unicode(n + 1, char_v)
					}
				} else {
					state = State::Error;
				}
			}
		}
	}
	String::from_utf8(buf).unwrap()
}

#[test]
fn test_from_utf16() {
	assert_eq!(
		from_utf16(&*RawValue::from_string("\"\\u00c2\\u00a9\"".to_string()).unwrap()),
		"©"
	);
}
