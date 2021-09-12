use cfg_if::cfg_if;
use web_sys::ServiceWorkerGlobalScope;
use wasm_bindgen::JsCast;
use worker::wasm_bindgen_futures::JsFuture;
use js_sys::Uint8Array;
use worker::console_log;
use std::num::ParseIntError;

pub struct JsCtx {
    ctx: ServiceWorkerGlobalScope
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

impl JsCtx {
    pub fn new() -> JsCtx {
        JsCtx {ctx: js_sys::global().dyn_into().unwrap()}
    }

    #[inline]
    fn get_crypto(&self) -> web_sys::SubtleCrypto{
        self.ctx.crypto().expect("Could not get crypto instance").subtle()
    }

    pub async fn get_key(&self, cf: &worker::Env) -> Result<web_sys::CryptoKey, &str>{
        // what the living fuck begins
        let usage = js_sys::JSON::parse(r#"["verify"]"#).unwrap();
        let algo = js_sys::JSON::parse(r#"{"name":"NODE-ED25519", "namedCurve": "NODE-ED25519"}"#).unwrap().dyn_into().unwrap();
        // what the living fuck ends
        let jwk = match cf.secret("DISCORD_PUBKEY") {
            Ok(key)  => js_sys::JSON::parse(&key.to_string()).unwrap().dyn_into().unwrap(),
            Err(_) => {
                return Err("Could not find DISCORD_PUBKEY")
            }
        };
        let key_promise = match self.get_crypto().import_key_with_object(
            "jwk",
            &jwk,
            &algo,
            false,
            &usage
        )  {
            Ok(promise) => JsFuture::from(promise).await,
            Err(err)    => {
                console_log!("Error importing key: {:?}", err);
                return Err("Error importing key")
            }
        };
        let key = match key_promise {
            Ok(js_key)  => match js_key.dyn_into() {
                Ok(key) => key,
                Err(err)    => {
                    console_log!("Error casting key: {:?}", err);
                    return   Err("Error casting key")
                }
            },
            Err(err)    => {
                console_log!("Error unwrapping key: {:?}", err);
                return Err("Error unwrapping key")
            }
        };
        Ok(key)
    }
    pub async fn verify_request(&self, key: &web_sys::CryptoKey, req: &worker::Request, body: Vec<u8>) -> Result<(), &str>{
        let headers = req.headers();
        let sig = match headers.get("x-signature-ed25519").unwrap() {
            Some(hdr) => {
                match decode_hex(&hdr) {
                    Ok(sig)  =>
                        {
                            let buf = Uint8Array::new_with_length(sig.len() as u32);
                            buf.copy_from(&sig);
                            buf
                        }
                    Err(msg) => {
                        console_log!("Failed to parse sig: {:?}", msg);
                        return Err("Failed to parse sig")
                    }
                }
            },
            None => return Err("Missing signature header")
        };
        let data = match headers.get("x-signature-timestamp").unwrap() {
            Some(hdr) => {
                let data = [hdr.as_bytes(), &body].concat();
                let buf = Uint8Array::new_with_length(data.len() as u32);
                buf.copy_from(&data);
                buf
            },
            None => return Err("Missing timestamp header")
        };
        console_log!("Sig: {:?}", sig.to_vec());
        console_log!("Data: {:?}", data.to_vec());
        match self.get_crypto().verify_with_object_and_buffer_source_and_buffer_source(
            &js_sys::JSON::parse(r#"{"name":"NODE-ED25519", "namedCurve": "NODE-ED25519"}"#).unwrap().dyn_into().unwrap(),
            key,
            &sig,
            &data,
        ){ 
            Ok(promise) => match JsFuture::from(promise).await {
                Ok(res) => match res.as_bool().unwrap(){
                    true  => Ok(()),
                    false => Err("Signature Mismatch")
                },
                Err(err) => {
                    console_log!("Error verifying: {:?}", err);
                    return Err("Error verifying")
                }
            }
            Err(err) => {
                console_log!("Error verifying: {:?}", err);
                return Err("Error verifying")
            }

        }
    }
}


cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}
