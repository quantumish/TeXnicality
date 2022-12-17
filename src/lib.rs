use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::console::log_3;
use web_sys::{HtmlElement, Document};
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;

static PROBLEMS: [(&'static str, &'static str, u64); 3]= [
	(r"e^{\pi i} = -1", "Euler's Identity", 1),
	(r"\frac{\mathrm{d}}{\mathrm{d}x} = \lim_{x\to \infty} \frac{f(x+h)-f(x)}{h} ", "Limit Definition of Derivative", 3),
	(r"A=\pi r^2", "Area of a Circle", 5),
];

static mut SCORE: u64 = 0;
static mut PTS_TO_GAIN: u64 = 0;
static mut SECONDS_LEFT: u64 = 20;
static mut TIMER_ID: u32 = 0;

#[wasm_bindgen]
extern "C" {
	fn setTimeout(f: &js_sys::Function, time: u32);
	fn setInterval(f: &js_sys::Function, time: u32) -> u32;
	fn clearInterval(id: u32);
}

pub fn toggle_element(document: &Document, id: &str, shown: bool) -> Result<(), JsValue> {
	let val = document.get_element_by_id(id).unwrap()
		.dyn_into::<web_sys::HtmlElement>().unwrap();
	val.set_hidden(!shown);
	Ok(())
}

pub fn get_element(document: &Document, id: &str) -> HtmlElement {
	document.get_element_by_id(id).unwrap()
		.dyn_into::<web_sys::HtmlElement>().unwrap()
}

fn clean_katex_html(html: String) -> String {
	let re = regex::Regex::new("<annotation.*/annotation>").unwrap();
	String::from(re.replace_all(&html, ""))
}

#[wasm_bindgen]
pub fn validate_problem() -> Result<(), JsValue> {
	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document on window");

	let textarea = document.get_element_by_id("user-input").unwrap()
		.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
	let val = textarea.value();

	let opts = katex::Opts::builder()
		.display_mode(true).throw_on_error(false).clone()
		.add_macro(String::from(r"\dv"), String::from(r"\frac{\mathrm{d}#1}{\mathrm{d}#2}"))
		.build().unwrap();
	let html = clean_katex_html(katex::render_with_opts(&val, &opts).unwrap());
	get_element(&document, "out").set_inner_html(&html);

	let target_html = get_element(&document, "target").inner_html();
	if target_html.trim().eq(html.trim()) {
		let parent = get_element(&document, "out").parent_element().unwrap();
		let classes = parent.class_name() + " correct";
		parent.set_class_name(&classes);
		textarea.set_disabled(true);

		unsafe {
			SCORE += PTS_TO_GAIN;
			let parent = get_element(&document, "score").set_inner_text(&SCORE.to_string());
		}

		let load = Closure::wrap(Box::new(move || { load_problem(); }) as Box<dyn FnMut()>);
		setTimeout(load.as_ref().unchecked_ref(), 1500);
		load.forget();
	}

	Ok(())
}

#[wasm_bindgen]
pub fn load_problem() -> Result<(), JsValue> {
	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document on window");

	get_element(&document, "out") .set_inner_html("");
	let parent = get_element(&document, "out").parent_element().unwrap();
	parent.set_class_name(
		&parent.class_name().split(' ')
			.filter(|c| !(*c).eq("correct"))
			.collect::<Vec<&str>>().join(" ")
	);

	let textarea = document.get_element_by_id("user-input").unwrap()
		.dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
	textarea.set_value("");
	textarea.set_disabled(false);

	let (eq, title, pts) = PROBLEMS.choose(&mut rand::thread_rng()).unwrap();

	get_element(&document, "problem-title").set_inner_text(title);
	get_element(&document, "problem-points").set_inner_text(&pts.to_string());
	unsafe { PTS_TO_GAIN = *pts; }

	let opts = katex::Opts::builder().display_mode(true).build().unwrap();
	let html = clean_katex_html(katex::render_with_opts(eq , &opts).unwrap());
	get_element(&document, "target").set_inner_html(&html);
	Ok(())
}


#[wasm_bindgen]
pub fn end_game() {
	unsafe { clearInterval(TIMER_ID); }
	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document on window");

	toggle_element(&document, "intro-window", false).unwrap();
	toggle_element(&document, "game-window", false).unwrap();
	toggle_element(&document, "ending-window", true).unwrap();

	unsafe {
		get_element(&document, "ending-text").set_inner_text(
			&format!("You finished with {} points!", SCORE)
		);
	}
}

#[wasm_bindgen]
pub fn start_game(timed: bool) -> Result<(), JsValue> {
	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document on window");

	toggle_element(&document, "intro-window", false)?;
	toggle_element(&document, "ending-window", false)?;
	toggle_element(&document, "game-window", true)?;

	let val = document.get_element_by_id("score").unwrap()
		.dyn_into::<web_sys::HtmlElement>()?
		.set_inner_text("0");

	load_problem()?;

	let time = Closure::wrap(Box::new(|| unsafe {
		let window = web_sys::window().expect("no global `window` exists");
		let document = window.document().expect("should have a document on window");
		SECONDS_LEFT -= 1;
		web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("called"));
		get_element(&document, "timer")
			.set_inner_text(
				&format!("{:02}:{:02}", SECONDS_LEFT / 60, SECONDS_LEFT % 60)
			);
		if SECONDS_LEFT == 0 {
			get_element(&document, "timer").set_inner_text("");
			end_game();
		}
	}) as Box<dyn FnMut()>);
	if timed {
		web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("bruh"));
		unsafe { TIMER_ID = setInterval(time.as_ref().unchecked_ref(), 1000); }
		time.forget();
	} else {
		get_element(&document, "timer").set_inner_html(&katex::render(r"\infty").unwrap());
	}
	Ok(())
}

// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
	// Use `web_sys`'s global `window` function to get a handle on the global
	// window object.
	let window = web_sys::window().expect("no global `window` exists");
	let document = window.document().expect("should have a document on window");

	toggle_element(&document, "ending-window", false)?;
	toggle_element(&document, "game-window", false)?;

	let start = Closure::wrap(Box::new(move || { start_game(true); }) as Box<dyn FnMut()>);
	let zen = Closure::wrap(Box::new(move || { start_game(false); }) as Box<dyn FnMut()>);
	let validate = Closure::wrap(Box::new(move || { validate_problem(); }) as Box<dyn FnMut()>);
	let load = Closure::wrap(Box::new(move || { load_problem(); }) as Box<dyn FnMut()>);
	let end = Closure::wrap(Box::new(move || { end_game(); }) as Box<dyn FnMut()>);
	web_sys::console::log_1(&wasm_bindgen::JsValue::from_str("wtfaxcs"));
	document.get_element_by_id("start-button-timed").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(start.as_ref().unchecked_ref()));
	document.get_element_by_id("start-button-untimed").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(zen.as_ref().unchecked_ref()));
	document.get_element_by_id("reset-button-timed").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(start.as_ref().unchecked_ref()));
	document.get_element_by_id("reset-button-untimed").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(zen.as_ref().unchecked_ref()));
	document.get_element_by_id("skip-button").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(load.as_ref().unchecked_ref()));
	document.get_element_by_id("end-game-button").unwrap()
		.dyn_into::<web_sys::HtmlButtonElement>().unwrap()
		.set_onclick(Some(end.as_ref().unchecked_ref()));
	let input = document.get_element_by_id("user-input").unwrap()
		.dyn_into::<web_sys::HtmlElement>().unwrap();
	input.set_onkeyup(Some(validate.as_ref().unchecked_ref()));

	start.forget();
	load.forget();
	validate.forget();
	end.forget();
	zen.forget();
	Ok(())
}
