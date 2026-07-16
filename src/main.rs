use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};

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
    
    let new_rp_title = use_state(|| String::new());
    
    let inc_rp_title = use_state(|| String::new());
    let inc_rp_words = use_state(|| String::new());

    let save_and_update = {
        let rps = rps.clone();
        Callback::from(move |new_data: HashMap<String, Roleplay>| {
            let _ = LocalStorage::set(CACHE_KEY, &new_data);
            rps.set(new_data);
        })
    };

    let on_new_title = {
        let new_rp_title = new_rp_title.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            new_rp_title.set(input.value());
        })
    };

    let on_select_change = {
        let inc_rp_title = inc_rp_title.clone();
        Callback::from(move |e: Event| {
            let select = e.target_unchecked_into::<HtmlSelectElement>();
            inc_rp_title.set(select.value());
        })
    };

    let on_inc_words = {
        let inc_rp_words = inc_rp_words.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            inc_rp_words.set(input.value());
        })
    };

    let create_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        let title = new_rp_title.clone();

        Callback::from(move |_| {
            let mut data = (*rps).clone();
            let t = (*title).clone();

            if !t.is_empty() && !data.contains_key(&t) {
                data.insert(t.clone(), Roleplay { title: t.clone(), total_words: 0, total_posts: 0 });
                save.emit(data);
                title.set(String::new());
            }
        })
    };

    let increment_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        let title = inc_rp_title.clone();
        let words = inc_rp_words.clone();

        Callback::from(move |_| {
            let mut data = (*rps).clone();
            let t = (*title).clone();
            let w: u32 = (*words).parse().unwrap_or(0);

            if !t.is_empty() && w > 0 {
                if let Some(rp) = data.get_mut(&t) {
                    rp.total_words += w;
                    rp.total_posts += 1;
                    save.emit(data);
                    words.set(String::new());
                }
            }
        })
    };

    let delete_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        Callback::from(move |title_to_delete: String| {
            let mut data = (*rps).clone();
            data.remove(&title_to_delete);
            save.emit(data);
        })
    };

    let edit_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        Callback::from(move |old_title: String| {
            let window = web_sys::window().unwrap();
            let mut data = (*rps).clone();

            if let Some(rp) = data.get(&old_title) {
                if let Some(new_title) = window.prompt_with_message_and_default("Nouveau titre :", &rp.title).unwrap_or(None) {
                    if !new_title.is_empty() {
                        if let Some(new_words_str) = window.prompt_with_message_and_default("Total de mots :", &rp.total_words.to_string()).unwrap_or(None) {
                            if let Some(new_posts_str) = window.prompt_with_message_and_default("Total de postes :", &rp.total_posts.to_string()).unwrap_or(None) {
                                
                                let new_words: u32 = new_words_str.parse().unwrap_or(rp.total_words);
                                let new_posts: u32 = new_posts_str.parse().unwrap_or(rp.total_posts);

                                let mut new_rp = rp.clone();
                                new_rp.title = new_title.clone();
                                new_rp.total_words = new_words;
                                new_rp.total_posts = new_posts;

                                data.remove(&old_title);
                                data.insert(new_title, new_rp);
                                save.emit(data);
                            }
                        }
                    }
                }
            }
        })
    };

    let global_words: u32 = rps.values().map(|rp| rp.total_words).sum();
    let global_posts: u32 = rps.values().map(|rp| rp.total_posts).sum();
    let global_avg = if global_posts == 0 { 0.0 } else { global_words as f64 / global_posts as f64 };

    let mut titles: Vec<String> = rps.keys().cloned().collect();
    titles.sort();

    html! {
        <div>
            <h1>{ "Tracker de RP" }</h1>
            
            <div class="card" style="background: #1a4d2e; border: 1px solid #2d7a47;">
                <h2>{ "Statistiques Globales" }</h2>
                <p>{ format!("Mots totaux : {} | Postes totaux : {}", global_words, global_posts) }</p>
                <p style="font-size: 1.2em; font-weight: bold;">
                    { format!("Moyenne Générale : {:.2} mots/poste", global_avg) }
                </p>
            </div>
            
            <div style="display: flex; gap: 15px; flex-wrap: wrap; margin-top: 20px;">
                <div class="card" style="flex: 1; min-width: 250px;">
                    <h3>{ "Créer un nouveau RP" }</h3>
                    <input 
                        placeholder="Titre du RP" 
                        value={(*new_rp_title).clone()}
                        oninput={on_new_title} 
                        style="width: 90%;"
                    />
                    <button onclick={create_rp} style="width: 90%; background: #007acc;">{ "Créer" }</button>
                </div>

                <div class="card" style="flex: 1; min-width: 250px;">
                    <h3>{ "Ajouter un poste" }</h3>
                    <select onchange={on_select_change} style="width: 90%; padding: 8px; margin: 5px 0; background: #2d2d2d; color: white;">
                        <option value="">{ "-- Sélectionner un RP --" }</option>
                        { for titles.iter().map(|k| html! { <option value={k.clone()} selected={*inc_rp_title == *k}>{k}</option> }) }
                    </select>
                    <input 
                        placeholder="Nombre de mots du poste" 
                        type="number"
                        value={(*inc_rp_words).clone()}
                        oninput={on_inc_words} 
                        style="width: 90%;"
                    />
                    <button onclick={increment_rp} style="width: 90%; background: #cc7a00;">{ "Incrémenter" }</button>
                </div>
            </div>

            <h2 style="margin-top: 30px;">{ "Vos RPs" }</h2>
            <div>
                { for rps.values().map(|rp| {
                    let title_del = rp.title.clone();
                    let title_edit = rp.title.clone();
                    
                    let cb_delete = {
                        let delete_rp = delete_rp.clone();
                        Callback::from(move |_| delete_rp.emit(title_del.clone()))
                    };
                    
                    let cb_edit = {
                        let edit_rp = edit_rp.clone();
                        Callback::from(move |_| edit_rp.emit(title_edit.clone()))
                    };

                    html! {
                        <div class="card" style="position: relative;">
                            <div style="position: absolute; right: 15px; top: 15px; display: flex; gap: 5px;">
                                <button onclick={cb_edit} style="background: #005999; padding: 5px 10px;">{ "✎" }</button>
                                <button onclick={cb_delete} style="background: #cc0000; padding: 5px 10px;">{ "X" }</button>
                            </div>
                            
                            <strong>{ &rp.title }</strong>
                            <p>{ format!("Postes: {} | Total: {} mots", rp.total_posts, rp.total_words) }</p>
                            <p style="color: #4da6ff;">{ format!("Moyenne: {:.2} mots/poste", rp.average()) }</p>
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