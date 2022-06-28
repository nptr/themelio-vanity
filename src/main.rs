use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

const HELP: &str = "\
Themelio Vanity Wallet Address Generator
USAGE:
  themelio-vanity [PATTERN]

PATTERN:
  The pattern which the wallet shall start with, excluding the 
  mandatory 't' prefix. Allowed characters are:

  0123456789abcdefghjkmnpqrstvwxyz. Unavailable are i,l,o and u.

  Characters `i,l,o` will be replaced with 1 or 0 respectively.
  `u` will be replaced with 'v' if it iss not the first char.
";

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		println!("{}", HELP);
		return;
	}

	if args[1].is_empty() {
		println!("{}", HELP);
		return;
	}

	let pattern = fix_pattern(&args[1]);
	if let Err(e) = pattern {
		println!("{}", e);
		return;
	}
	let mut addr = "t".to_owned();
	addr.push_str(&pattern.ok().unwrap());

	let mut handles: Vec<JoinHandle<_>> = Vec::new();
	let can_run_flag = Arc::new(AtomicBool::new(true));

	for _ in 0..4 {
		let thread_can_run = can_run_flag.clone();
		let thread_addr = addr.clone();

		let handle = thread::spawn(move || {
			let mut wallet_contract: Vec<u8> = vec![
				// OpCode::LoadImm(HADDR_SPENDER_INDEX)
				0x42, 0x00, 0x09, 
				// OpCode::PushI(6u32.into())
				0xF1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x06, 
				// OpCode::LoadImm(HADDR_SPENDER_TX)
				0x42, 0x00, 0x00, 
				// OpCode::VRef
				0x50, 
				// OpCode::VRef
				0x50, 
				// OpCode::PushB(public_key.to_bytes().)
				0xF0, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
				0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
				// OpCode::LoadImm(1)
				0x42, 0x00, 0x01, 
				// OpCode::SigEOk(32)
				0x32, 0x00, 0x20,
			];
			while thread_can_run.load(Ordering::Relaxed) {
				let (pk, sk) = tmelcrypt::ed25519_keygen();
				wallet_contract[43..75].copy_from_slice(&pk.0);
				let hv = tmelcrypt::hash_single(&wallet_contract);
				let gen_addr = hv.to_addr();
				if gen_addr.starts_with(&thread_addr) {
					let sk_phrase = base32::encode(base32::Alphabet::Crockford, &sk.0[..32]);
					println!("Address: {}", &gen_addr);
					println!("Secret Phrase: {}", sk_phrase);
					thread_can_run.store(false, Ordering::Relaxed);
					break;
				}
			}
		});

		handles.push(handle);
	}

	for handle in handles {
		handle.join().expect("Couldn't join threads!");
	}
}

fn alpha_to_num(char: char) -> Option<char> {
	match char {
		'0' => Some('0'),
		'1' => Some('1'),
		'2' => Some('2'),
		'3' => Some('3'),
		'4' => Some('4'),
		'5' => Some('5'),
		'6' => Some('6'),
		'7' => Some('7'),
		'8' => Some('8'),
		'9' => Some('9'),
		'a' => Some('4'),
		'b' => Some('8'),
		'e' => Some('3'),
		'g' => Some('6'),
		'h' => Some('4'),
		'i' => Some('1'),
		'l' => Some('1'),
		'o' => Some('0'),
		'p' => Some('9'),
		'q' => Some('0'),
		'r' => Some('2'),
		's' => Some('5'),
		't' => Some('7'),
		'z' => Some('2'),
		_ => None,
	}
}

fn un_crockford(char: char) -> Option<char> {
	match char {
		'i' => Some('1'),
		'l' => Some('1'),
		'o' => Some('0'),
		'u' => Some('v'),
		_ => None,
	}
}

fn fix_pattern(str: &String) -> Result<String, String> {
	let lcstr = str.to_lowercase();
	let mut pattern = String::with_capacity(lcstr.len());

	// First character after prefix must be a number. It stems from the
	// checksum being appended to the public key before base32'ing.
	let fc = lcstr.chars().nth(0).unwrap();
	let fcnew = alpha_to_num(fc);
	if fcnew.is_none() {
		return Err(format!(
			"First character must be a number. No substitute found for '{}'.",
			fc
		));
	}

	if fcnew.unwrap() != fc {
		println!(
			"Invalid first character '{}' in wallet address replaced with '{}'.",
			fc,
			fcnew.unwrap()
		);
	}

	pattern.push(fcnew.unwrap());

	// All other characters can be from the full Crockford alphabet.
	// We only need to replace the invalid ones if possible.
	let crockford_alphabet = vec![
		'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
		'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z',
	];

	let mut it = lcstr.chars();
	it.next();
	for c in it {
		if !crockford_alphabet.contains(&c) {
			let cnew = un_crockford(c);
			if cnew.is_none() {
				return Err(format!("Wallet address can't contain character '{}'.", c));
			} else {
				println!(
					"Invalid character '{}' in wallet address replaced with '{}'.",
					c,
					cnew.unwrap()
				);
				pattern.push(cnew.unwrap());
			}
		} else {
			pattern.push(c);
		}
	}

	return Ok(pattern);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_pattern_recover() {
		assert_eq!(fix_pattern(&String::from("aaa")).unwrap(), "4aa");
		assert_eq!(fix_pattern(&String::from("bbb")).unwrap(), "8bb");
		assert_eq!(fix_pattern(&String::from("111")).unwrap(), "111");
		assert_eq!(fix_pattern(&String::from("lib")).unwrap(), "11b");
		assert_eq!(fix_pattern(&String::from("lob")).unwrap(), "10b");
    }

	#[test]
	fn fix_pattern_recover_uppercase() {
		assert_eq!(fix_pattern(&String::from("Aaa")).unwrap(), "4aa");
		assert_eq!(fix_pattern(&String::from("bBb")).unwrap(), "8bb");
    }

	#[test]
	fn fix_pattern_invalid() {
		assert_eq!(fix_pattern(&String::from("uuu")).is_err(), true);
		assert_eq!(fix_pattern(&String::from("1uu")).is_err(), false);
		assert_eq!(fix_pattern(&String::from("11u")).is_err(), false);
    }
}
