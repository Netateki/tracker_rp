use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement};

const CACHE_KEY: &str = "rp_tracker_data";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Category {
    Normal,
    Duo,
    Elevage,
    DuoElevage, // Nouvelle catégorie combinée
}

impl Default for Category {
    fn default() -> Self {
        Category::Normal
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Roleplay {
    pub title: String,
    pub total_words: u32,
    pub total_posts: u32,
    #[serde(default)]
    pub category: Category,
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
    let new_rp_category = use_state(|| Category::Normal);
    
    let inc_rp_title = use_state(|| String::new());
    let inc_rp_words = use_state(|| String::new());

    let export_text = use_state(|| String::new());

    let is_staff = use_state(|| false);
    let is_jury = use_state(|| false);

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

    let on_new_category = {
        let new_rp_category = new_rp_category.clone();
        Callback::from(move |e: Event| {
            let select = e.target_unchecked_into::<HtmlSelectElement>();
            let cat = match select.value().as_str() {
                "Duo" => Category::Duo,
                "Elevage" => Category::Elevage,
                "DuoElevage" => Category::DuoElevage,
                _ => Category::Normal,
            };
            new_rp_category.set(cat);
        })
    };

    let create_rp = {
        let rps = rps.clone();
        let save = save_and_update.clone();
        let title = new_rp_title.clone();
        let category = new_rp_category.clone();

        Callback::from(move |_| {
            let mut data = (*rps).clone();
            let t = (*title).clone();

            if !t.is_empty() && !data.contains_key(&t) {
                data.insert(t.clone(), Roleplay { 
                    title: t.clone(), 
                    total_words: 0, 
                    total_posts: 0,
                    category: (*category).clone()
                });
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

    let edit_rp_stats = {
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
                                let mut new_rp = rp.clone();
                                new_rp.title = new_title.clone();
                                new_rp.total_words = new_words_str.parse().unwrap_or(rp.total_words);
                                new_rp.total_posts = new_posts_str.parse().unwrap_or(rp.total_posts);

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

    // --- LOGIQUE DE CALCUL ---
    let global_words: u32 = rps.values().map(|rp| rp.total_words).sum();
    let global_posts: u32 = rps.values().map(|rp| rp.total_posts).sum();
    let global_avg = if global_posts == 0 { 0.0 } else { global_words as f64 / global_posts as f64 };

    let mut duo_posts = 0;
    let mut elevage_posts = 0;

    for rp in rps.values() {
        match rp.category {
            Category::Duo => duo_posts += rp.total_posts,
            Category::Elevage => elevage_posts += rp.total_posts,
            Category::DuoElevage => {
                duo_posts += rp.total_posts;
                elevage_posts += rp.total_posts;
            }
            Category::Normal => {}
        }
    }

    let lvl_base = match global_posts {
        p if p >= 30 => 12,
        p if p >= 20 => 9,
        p if p >= 10 => 6,
        p if p >= 2 => 3,
        _ => 0,
    };

    let lvl_mots = if global_posts < 2 {
        0
    } else if global_avg >= 1000.0 {
        if global_posts >= 20 { 4 } else { 2 }
    } else if global_avg >= 700.0 {
        if global_posts >= 20 { 2 } else { 1 }
    } else {
        0
    };

    let lvl_duo = if duo_posts >= 2 { 1 } else { 0 };
    let lvl_staff = if *is_staff { 2 } else { 0 };
    let lvl_jury = if *is_jury { 2 } else { 0 };

    let total_niveaux_bonus = lvl_base + lvl_mots + lvl_duo + lvl_staff + lvl_jury;

    let lvl_elevage = match elevage_posts {
        e if e >= 21 => 5,
        e if e >= 16 => 4,
        e if e >= 10 => 3,
        e if e >= 5 => 2,
        e if e >= 1 => 1,
        _ => 0,
    };

    // --- GÉNÉRATION DU BBCODE ---
    let generate_bbcode = {
        let rps = rps.clone();
        let export_text = export_text.clone();
        Callback::from(move |_| {
            let mut output = String::from("[b]Mois[/b]\n[list]\n");
            let mut titles: Vec<&String> = rps.keys().collect();
            titles.sort();

            for title in titles {
                if let Some(rp) = rps.get(title) {
                    let cat_tag = match rp.category {
                        Category::Duo => " [Duo]",
                        Category::Elevage => " [Élevage]",
                        Category::DuoElevage => " [Duo] [Élevage]",
                        Category::Normal => "",
                    };
                    output.push_str(&format!("[*] [url=LIEN_ICI]{}{}[/url] ; {} RP(s)\n", rp.title, cat_tag, rp.total_posts));
                }
            }

            output.push_str(&format!("[b]Total :[/b] {} mots\n", global_words));
            output.push_str(&format!("[b]Moyenne de mots/rp :[/b] {:.2}\n[/list]\n", global_avg));
            
            output.push_str("\n[b]-- RÉCOMPENSES --[/b]\n");
            output.push_str(&format!("[b]Niveaux Bonus :[/b] {} (Base: {}, Mots: {}, Duo: {}, Staff: {}, Jury: {})\n", 
                total_niveaux_bonus, lvl_base, lvl_mots, lvl_duo, lvl_staff, lvl_jury));
            
            if lvl_elevage > 0 {
                output.push_str(&format!("[b]Niveaux Élevage (à distribuer) :[/b] {}\n", lvl_elevage));
            }

            export_text.set(output);
        })
    };

    let mut titles: Vec<String> = rps.keys().cloned().collect();
    titles.sort();

    html! {
        <div style="max-width: 900px; margin: 0 auto;">
            <h1>{ "Tracker de RP & Niveaux" }</h1>
            
            <div style="display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 20px;">
                <div class="card" style="flex: 1; background: #1a4d2e; border: 1px solid #2d7a47;">
                    <h2>{ "Statistiques" }</h2>
                    <p>{ format!("Postes totaux : {} | Mots : {}", global_posts, global_words) }</p>
                    <p>{ format!("RP Duo : {} | RP Élevage : {}", duo_posts, elevage_posts) }</p>
                    <p style="font-size: 1.2em; font-weight: bold; color: #4da6ff;">
                        { format!("Moyenne : {:.2} mots/rp", global_avg) }
                    </p>
                </div>

                <div class="card" style="flex: 1; background: #4a2e1a; border: 1px solid #7a472d;">
                    <h2>{ "Niveaux Gagnés" }</h2>
                    <p style="font-size: 1.5em; margin: 5px 0; color: #ffcc00; font-weight: bold;">
                        { format!("Niveaux Bonus : +{}", total_niveaux_bonus) }
                    </p>
                    <p style="font-size: 0.9em; color: #ccc;">
                        { format!("(Base: {}, Mots: {}, Duo: {}, Staff: {}, Jury: {})", lvl_base, lvl_mots, lvl_duo, lvl_staff, lvl_jury) }
                    </p>
                    <p style="font-size: 1.2em; margin-top: 10px; color: #66ff66; font-weight: bold;">
                        { format!("Niveaux Élevage : +{}", lvl_elevage) }
                    </p>
                </div>
            </div>

            <div class="card" style="background: #2a2a2a; border: 1px solid #444; margin-bottom: 20px;">
                <h3>{ "Options Globales (Bonus fixes)" }</h3>
                <label style="margin-right: 20px; cursor: pointer;">
                    <input type="checkbox" checked={*is_staff} onchange={
                        let is_staff = is_staff.clone();
                        Callback::from(move |e: Event| is_staff.set(e.target_unchecked_into::<HtmlInputElement>().checked()))
                    } />
                    { " Membre du Staff (+2)" }
                </label>
                <label style="cursor: pointer;">
                    <input type="checkbox" checked={*is_jury} onchange={
                        let is_jury = is_jury.clone();
                        Callback::from(move |e: Event| is_jury.set(e.target_unchecked_into::<HtmlInputElement>().checked()))
                    } />
                    { " Participation Jury (+2)" }
                </label>
            </div>
            
            <div style="display: flex; gap: 15px; flex-wrap: wrap;">
                <div class="card" style="flex: 1; min-width: 250px;">
                    <h3>{ "Créer un nouveau RP" }</h3>
                    <input placeholder="Titre du RP" value={(*new_rp_title).clone()} oninput={on_new_title} style="width: 90%; margin-bottom: 5px;" />
                    <select onchange={on_new_category} style="width: 90%; padding: 8px; margin-bottom: 10px; background: #2d2d2d; color: white;">
                        <option value="Normal">{ "Normal" }</option>
                        <option value="Duo">{ "Duo" }</option>
                        <option value="Elevage">{ "Élevage" }</option>
                        <option value="DuoElevage">{ "Duo & Élevage" }</option>
                    </select>
                    <button onclick={create_rp} style="width: 90%; background: #007acc;">{ "Créer" }</button>
                </div>

                <div class="card" style="flex: 1; min-width: 250px;">
                    <h3>{ "Ajouter un poste" }</h3>
                    <select onchange={
                        let inc_rp_title = inc_rp_title.clone();
                        Callback::from(move |e: Event| inc_rp_title.set(e.target_unchecked_into::<HtmlSelectElement>().value()))
                    } style="width: 90%; padding: 8px; margin: 5px 0; background: #2d2d2d; color: white;">
                        <option value="">{ "-- Sélectionner un RP --" }</option>
                        { for titles.iter().map(|k| html! { <option value={k.clone()} selected={*inc_rp_title == *k}>{k}</option> }) }
                    </select>
                    <input placeholder="Mots du poste" type="number" value={(*inc_rp_words).clone()} oninput={
                        let inc_rp_words = inc_rp_words.clone();
                        Callback::from(move |e: InputEvent| inc_rp_words.set(e.target_unchecked_into::<HtmlInputElement>().value()))
                    } style="width: 90%; margin-bottom: 10px;" />
                    <button onclick={increment_rp} style="width: 90%; background: #cc7a00;">{ "Incrémenter" }</button>
                </div>
            </div>

            <div class="card" style="margin-top: 20px; background: #2a2a35; border: 1px solid #555;">
                <h2>{ "Export Forum (BBCode)" }</h2>
                <button onclick={generate_bbcode} style="background: #800080; margin-bottom: 10px;">{ "Générer BBCode avec Récompenses" }</button>
                <br/>
                <textarea readonly=true value={(*export_text).clone()} style="width: 95%; height: 200px; background: #1e1e1e; color: #fff; font-family: monospace; resize: vertical; padding: 10px;" />
            </div>

            <h2 style="margin-top: 30px;">{ "Vos RPs Actifs" }</h2>
            <div>
                { for rps.values().map(|rp| {
                    let title_del = rp.title.clone();
                    let title_edit = rp.title.clone();
                    let title_cat = rp.title.clone();
                    
                    let cb_delete = {
                        let delete_rp = delete_rp.clone();
                        Callback::from(move |_| delete_rp.emit(title_del.clone()))
                    };
                    
                    let cb_edit = {
                        let edit_rp_stats = edit_rp_stats.clone();
                        Callback::from(move |_| edit_rp_stats.emit(title_edit.clone()))
                    };

                    let cb_cat_change = {
                        let rps = rps.clone();
                        let save = save_and_update.clone();
                        Callback::from(move |e: Event| {
                            let select = e.target_unchecked_into::<HtmlSelectElement>();
                            let cat = match select.value().as_str() {
                                "Duo" => Category::Duo,
                                "Elevage" => Category::Elevage,
                                "DuoElevage" => Category::DuoElevage,
                                _ => Category::Normal,
                            };
                            let mut data = (*rps).clone();
                            if let Some(rp_mut) = data.get_mut(&title_cat) {
                                rp_mut.category = cat;
                                save.emit(data);
                            }
                        })
                    };

                    let cat_color = match rp.category {
                        Category::Duo => "#ff66b2",
                        Category::Elevage => "#66ff66",
                        Category::DuoElevage => "#cc66ff", // Couleur violette pour l'état hybride
                        Category::Normal => "#ccc",
                    };

                    html! {
                        <div class="card" style="position: relative; margin-bottom: 10px;">
                            <div style="position: absolute; right: 15px; top: 15px; display: flex; gap: 5px;">
                                <button onclick={cb_edit} style="background: #005999; padding: 5px 10px;">{ "✎ Stats" }</button>
                                <button onclick={cb_delete} style="background: #cc0000; padding: 5px 10px;">{ "X" }</button>
                            </div>
                            
                            <strong style="font-size: 1.2em;">{ &rp.title }</strong>
                            
                            <div style="margin: 10px 0;">
                                <span style="margin-right: 10px;">{ "Catégorie :" }</span>
                                <select onchange={cb_cat_change} style={format!("padding: 5px; background: #222; color: {}; font-weight: bold; border: 1px solid {};", cat_color, cat_color)}>
                                    <option value="Normal" selected={rp.category == Category::Normal}>{ "Normal" }</option>
                                    <option value="Duo" selected={rp.category == Category::Duo}>{ "Duo" }</option>
                                    <option value="Elevage" selected={rp.category == Category::Elevage}>{ "Élevage" }</option>
                                    <option value="DuoElevage" selected={rp.category == Category::DuoElevage}>{ "Duo & Élevage" }</option>
                                </select>
                            </div>

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