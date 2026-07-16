use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yew::prelude::*;
use web_sys::HtmlInputElement;

const CACHE_KEY: &str = "rp_tracker_data";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Roleplay {
    pub title: String,
    pub total_words: u32,
    pub total_posts: u32,
}

impl Roleplay {
    pub fn average(&self) -> f64 {
        if self.total_posts == 0 { return 0.0; }
        self.total_words as f64 / self.total_posts as f64
    }
}

#[function_component(App)]
fn app() -> Html {
    let rps = use_state(|| LocalStorage::get::<HashMap<String, Roleplay>>(CACHE_KEY).unwrap_or_default());
    let input_title = use_state(|| String::new());
    let input_words = use_state(|| String::new());

    let save_and_update = {
        let rps = rps.clone();
        Callback::from(move |new_data: HashMap<String, Roleplay>| {
            let _ = LocalStorage::set(CACHE_KEY, &new_data);
            rps.set(new_data);
        })
    };

    // --- EXTRACTION DE LA LOGIQUE POUR PROTÉGER LA MACRO HTML ---
    
    let on_title_change = {
        let input_title = input_title.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            input_title.set(input.value());
        })
    };

    let on_words_change = {
        let input_words = input_words.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            input_words.set(input.value());
        })
    };

    let on_submit = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        let title = input_title.clone();
        let words = input_words.clone();

        Callback::from(move |_| {
            let mut data = (*rps).clone();
            let t = (*title).clone();
            let w: u32 = (*words).parse().unwrap_or(0);

            if !t.is_empty() {
                let rp = data.entry(t.clone()).or_insert(Roleplay {
                    title: t.clone(),
                    total_words: 0,
                    total_posts: 0,
                });
                
                rp.total_words += w;
                rp.total_posts += 1;

                save.emit(data);
                
                title.set(String::new());
                words.set(String::new());
            }
        })
    };

    // Fonction de suppression isolée
    let delete_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        Callback::from(move |title_to_delete: String| {
            let mut data = (*rps).clone();
            data.remove(&title_to_delete);
            save.emit(data);
        })
    };

    // --- RENDU GRAPHIQUE ---

    html! {
        <div>
            <h1>{ "Tracker de RP" }</h1>
            
            <div class="card">
                <h3>{ "Ajouter un Poste (ou Créer un RP)" }</h3>
                <input 
                    placeholder="Titre du RP" 
                    value={(*input_title).clone()}
                    oninput={on_title_change} 
                />
                <br/>
                <input 
                    placeholder="Nombre de mots" 
                    type="number"
                    value={(*input_words).clone()}
                    oninput={on_words_change} 
                />
                <br/>
                <button onclick={on_submit}>{ "Valider le poste" }</button>
            </div>

            <h2>{ "Vos Statistiques" }</h2>
            <div>
                { for rps.values().map(|rp| {
                    let title = rp.title.clone();
                    let delete_cb = {
                        let delete_rp = delete_rp.clone();
                        let t = title.clone();
                        Callback::from(move |_| delete_rp.emit(t.clone()))
                    };

                    html! {
                        <div class="card" style="position: relative;">
                            // Le bouton de suppression
                            <button 
                                onclick={delete_cb} 
                                style="position: absolute; right: 15px; top: 15px; background: #cc0000;"
                            >
                                { "X" }
                            </button>
                            
                            <strong>{ &title }</strong>
                            <p>{ format!("Postes: {} | Total: {} mots", rp.total_posts, rp.total_words) }</p>
                            <p style="color: #007acc;">{ format!("Moyenne: {:.2} mots/poste", rp.average()) }</p>
                        </div>
                    }
                }) }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}