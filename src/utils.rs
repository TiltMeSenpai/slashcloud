use cfg_if::cfg_if;
use web_sys::ServiceWorkerGlobalScope;
use wasm_bindgen::JsCast;
use worker::wasm_bindgen_futures::JsFuture;
use js_sys::{Uint8Array,JsString};
use worker::console_log;
use std::convert::TryInto;

pub struct JsCtx {
    ctx: ServiceWorkerGlobalScope
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

    async fn verify_buffer(&self, key: &web_sys::CryptoKey, sig: Uint8Array, data: Uint8Array) -> Result<(), &str>{
        match self.get_crypto().verify_with_object_and_buffer_source_and_buffer_source(
            &js_sys::JSON::parse(r#"{"name":"NODE-ED25519", "namedCurve": "NODE-ED25519"}"#).unwrap().dyn_into().unwrap(),
            key,
            &sig,
            &data,
        ){ 
            Ok(promise) => match JsFuture::from(promise).await {
                Ok(res) => match res.as_bool().unwrap_or(false){
                    true  => Ok(()),
                    false => Err("Signature mismatch")
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

    pub async fn verify_request(&self, key: &web_sys::CryptoKey, req: &worker::Request, body: String) -> Result<(), &str>{
        let headers = req.headers();
        let sig = match headers.get("X-Signature-Ed25519").unwrap() {
            Some(hdr) => {
                match hex::decode(hdr) {
                    Ok(sig)  => unsafe { Uint8Array::view(&sig) }
                    Err(msg) => {
                        console_log!("Failed to parse sig: {:?}", msg);
                        return Err("Failed to parse sig")
                    }
                }
            },
            None => return Err("Missing signature header")
        };
        console_log!("Signature len: {:?}", sig.length());
        let data = match headers.get("X-Signature-Timestamp").unwrap() {
            Some(mut hdr) => {
                hdr.push_str(&body);
                unsafe { Uint8Array::view(hdr.as_bytes())}
            },
            None => return Err("Missing timestamp header")
        };
        console_log!("Calling verify");
        self.verify_buffer(key, sig, data).await
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
