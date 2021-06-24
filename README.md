# Vk-auth.

Crate allows you to retrieve access token to API with just phone/email, password and app_id;

## Example
```
➜  vk_auth git:(master) ✗ APP_ID=YOUR_APP_ID EMAIL=YOUR_PHONE_OR_EMAIL PASSWORD=PASSWORD cargo run --example main
   Compiling vk-auth v0.1.0 (/home/anton/Projects/vk_auth)
    Finished dev [unoptimized + debuginfo] target(s) in 3.00s
     Running `target/debug/examples/main`
AccessToken { access_token: "token", expires_in: 86400s, user_id: "user_id" }
```