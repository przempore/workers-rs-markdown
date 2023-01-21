use pulldown_cmark::{html, Options, Parser};
use serde_json::json;
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

fn get_weather_info(req: &Request) -> String {
    let html_style = "body{padding:6em; font-family: sans-serif;} h1{color:#f6821f;}";

    let mut html_content = format!("<p> Colo: {} </p>", req.cf().colo());
    html_content = format!(
        "{}<p> Country: {} </p>",
        html_content,
        req.cf().country().unwrap_or("unknown country".into())
    );
    html_content = format!(
        "{}<p> City: {} </p>",
        html_content,
        req.cf().city().unwrap_or("unknown city".into())
    );
    html_content = format!(
        "{}<p> Continent: {} </p>",
        html_content,
        req.cf().continent().unwrap_or("unknown continent".into())
    );
    html_content = format!(
        "{}<p> Latitude: {} </p>",
        html_content,
        req.cf().coordinates().unwrap_or((0f32, 0f32)).0
    );
    html_content = format!(
        "{}<p> Longitude: {} </p>",
        html_content,
        req.cf().coordinates().unwrap_or((0f32, 0f32)).1
    );
    html_content = format!(
        "{}<p> PostalCode: {} </p>",
        html_content,
        req.cf()
            .postal_code()
            .unwrap_or("unknown postalCode".into())
    );
    html_content = format!(
        "{}<p> MetroCode: {} </p>",
        html_content,
        req.cf().metro_code().unwrap_or("unknown metroCode".into())
    );
    html_content = format!(
        "{}<p> Region: {} </p>",
        html_content,
        req.cf().region().unwrap_or("unknown region".into())
    );
    html_content = format!(
        "{}<p> RegionCode: {} </p>",
        html_content,
        req.cf()
            .region_code()
            .unwrap_or("unknown regionCode".into())
    );

    format!(
        "<!DOCTYPE html>
        <head>
        <title> Geolocation: Hello World </title>
        <style> ${html_style} </style>
        </head>
        <body>
        <h1>Geolocation: Hello World!</h1>
        <p>You now have access to geolocation data about where your user is visiting from.</p>
        {}
    </body>",
        html_content
    )
}

#[warn(dead_code)]
fn parse() -> String {
    let markdown_input: &str = "Hello world, this is a ~~complicated~~ *very simple* example.";
    println!("Parsing the following Markdown string:\n{}", markdown_input);

    // Set up options and parser. Strikethroughs are not part of the CommonMark standard
    // and we therefore must enable it explicitly.
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(markdown_input, options);

    // Write to String buffer.
    let mut html_output: String = String::with_capacity(markdown_input.len() * 3 / 2);
    html::push_html(&mut html_output, parser);

    // Check that the output is what you expected.
    let expected_html: &str =
        "<p>Hello world, this is a <del>complicated</del> <em>very simple</em> example.</p>\n";
    assert_eq!(expected_html, &html_output);

    format!("\nHTML output:\n{}", &html_output)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    let _ = get_weather_info(&req);
    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    let res = router
        .get("/", |req, _| Response::ok(get_weather_info(&req)))
        .post_async("/form/:field", |mut req, ctx| async move {
            if let Some(name) = ctx.param("field") {
                let form = req.form_data().await?;
                match form.get(name) {
                    Some(FormEntry::Field(value)) => {
                        return Response::from_json(&json!({ name: value }))
                    }
                    Some(FormEntry::File(_)) => {
                        return Response::error("`field` param in form shouldn't be a File", 422);
                    }
                    None => return Response::error("Bad Request", 400),
                }
            }

            Response::error("Bad Request", 400)
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await?;

    let mut headers = Headers::new();
    let _ = headers.set("Content-type", "text/html;charset=UTF-8");
    let r = res.with_headers(headers);
    Ok(r)
}
