#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
struct Measurement {
    value: f64,
    label: String,
}

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let measurements = vec![
        Measurement {
            value: 20.9,
            label: "Nedervåning bord".to_owned(),
        },
        Measurement {
            value: 16.9,
            label: "Ute".to_owned(),
        },
        Measurement {
            value: -14.8,
            label: "Frysen".to_owned(),
        },
        Measurement {
            value: 22.5,
            label: "Nedervåning tak".to_owned(),
        },
        Measurement {
            value: 23.6,
            label: "Tv-rum tak".to_owned(),
        },
        Measurement {
            value: 308.0,
            label: "Elpris nu".to_owned(),
        },
        Measurement {
            value: 311.0,
            label: "Elpris nästa timme".to_owned(),
        },
    ];

    cx.render(rsx! {
        div {
            //class: "flex flex-row flex-wrap gap-5 m-5 bg-black text-sky-500",
            class: "bubbles",

            for m in &measurements {
                Bubble {
                    measurement: m.clone(),
                }

            }
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}
            // Bubble {}

        }
    })
}

#[inline_props]
fn Bubble(cx: Scope, measurement: Measurement) -> Element {
    let Measurement { value, label } = measurement;
    // dbg!(value, label);
    render! {
        div {
            //class: "shrink-0 self-auto flex items-center justify-center rounded-xl bg-slate-950 w-40 h-40 text-sky-500",
            class: "bubble",

            div {
                class: "value",

                "{value}"
            }

            div {
                class: "label",

                "{label}"
            }
        }
    }
}
