use super::{ConfigEditor, TemplateResult};
use anyhow::Result;
use xmltree::Element;

impl ConfigEditor {
    /// Apply a room template — creates standard controls for a given room type.
    /// Returns a human-readable summary of what was created.
    /// `add_fn` is called for each block: (block_type, title, room_uuid, cat_uuid) → Result<uuid>
    pub fn apply_template(&mut self, template: &str, room: &str) -> Result<TemplateResult> {
        // Verify room exists
        let room_uuid = self.find_room_uuid(room)?;

        // Template definitions: (type, title_suffix, category, params)
        struct Tmpl {
            block_type: &'static str,
            title_suffix: &'static str,
            category: &'static str,
            params: Vec<(&'static str, &'static str)>,
        }

        let blocks: Vec<Tmpl> = match template.to_lowercase().as_str() {
            "standard" | "wohnzimmer" | "living" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "1"), ("MoveTimeout", "600")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
                Tmpl {
                    block_type: "Thermostat",
                    title_suffix: "Klima",
                    category: "Raumklima",
                    params: vec![("TargetTemp", "21"), ("Hysteresis", "0.5")],
                },
            ],
            "bedroom" | "schlafzimmer" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "2"), ("MoveTimeout", "300")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
                Tmpl {
                    block_type: "AlarmClock",
                    title_suffix: "Wecker",
                    category: "Beleuchtung",
                    params: vec![],
                },
            ],
            "bathroom" | "badezimmer" | "bad" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "1"), ("MoveTimeout", "300")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
                Tmpl {
                    block_type: "Thermostat",
                    title_suffix: "Klima",
                    category: "Raumklima",
                    params: vec![("TargetTemp", "23"), ("Hysteresis", "0.5")],
                },
                Tmpl {
                    block_type: "Ventilation2",
                    title_suffix: "Lüftung",
                    category: "Raumklima",
                    params: vec![],
                },
            ],
            "hallway" | "flur" | "gang" | "diele" => vec![Tmpl {
                block_type: "LightController2",
                title_suffix: "Licht",
                category: "Beleuchtung",
                params: vec![("FadingTime", "0.5"), ("MoveTimeout", "120")],
            }],
            "kitchen" | "küche" | "kueche" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "0.5"), ("MoveTimeout", "600")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
                Tmpl {
                    block_type: "Thermostat",
                    title_suffix: "Klima",
                    category: "Raumklima",
                    params: vec![("TargetTemp", "21"), ("Hysteresis", "0.5")],
                },
            ],
            "outdoor" | "aussen" | "terrasse" | "garten" | "balkon" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "0.5"), ("MoveTimeout", "300")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
            ],
            "office" | "büro" | "buero" | "arbeitszimmer" => vec![
                Tmpl {
                    block_type: "LightController2",
                    title_suffix: "Licht",
                    category: "Beleuchtung",
                    params: vec![("FadingTime", "1"), ("MoveTimeout", "600")],
                },
                Tmpl {
                    block_type: "JalousieUpDown2",
                    title_suffix: "Beschattung",
                    category: "Beschattung",
                    params: vec![("UpTime", "60"), ("DownTime", "60")],
                },
                Tmpl {
                    block_type: "Thermostat",
                    title_suffix: "Klima",
                    category: "Raumklima",
                    params: vec![("TargetTemp", "22"), ("Hysteresis", "0.5")],
                },
            ],
            other => {
                let available = [
                    "standard", "bedroom", "bathroom", "hallway", "kitchen", "outdoor", "office",
                ];
                return Err(crate::errors::not_found_error(
                    "Template",
                    other,
                    &available.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                    "Available: standard, bedroom, bathroom, hallway, kitchen, outdoor, office",
                ));
            }
        };

        // Return the plan: vec of (block_type, title, params)
        let mut plan = Vec::new();
        for block in blocks {
            let title = format!("{} {}", room, block.title_suffix);
            let cat_uuid = match self.find_category_uuid(block.category) {
                Ok(uuid) => uuid,
                Err(_) => self.add_category(block.category)?,
            };
            plan.push((
                block.block_type.to_string(),
                title,
                block
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            ));
            // Store cat_uuid + room_uuid in the returned data for caller to use
            // Actually, let's just do it inline since we have &mut self
            let title_ref = plan.last().unwrap().1.clone();
            let block_type = block.block_type;

            // Find parent for this type (Page for logic/controller types, root otherwise)
            let parent = self.find_auto_parent(block_type);

            if let Some(parent_sel) = parent {
                self.add_element(
                    &parent_sel,
                    block_type,
                    &title_ref,
                    None,
                    Some(&room_uuid),
                    Some(&cat_uuid),
                    &[],
                )?;
            } else {
                self.add_element_to_root(
                    block_type,
                    &title_ref,
                    Some(&room_uuid),
                    Some(&cat_uuid),
                    &[],
                )?;
            }

            // Set parameters
            for (key, val) in &block.params {
                let _ = self.set_param(&title_ref, key, val);
            }
        }

        Ok(plan)
    }

    /// Find the auto-parent selector for a block type.
    fn find_auto_parent(&self, block_type: &str) -> Option<String> {
        // Page-level types go under first Page
        let page_types = [
            "LightController2",
            "JalousieUpDown2",
            "Thermostat",
            "AlarmClock",
            "Ventilation2",
            "And",
            "Or",
            "Not",
            "Xor",
            "FlipFlop",
            "AMemory",
            "Counter",
            "State",
            "InputRef",
            "OutputRef",
        ];
        if page_types.contains(&block_type) {
            // Find first Page
            fn find_page(elem: &Element) -> Option<String> {
                if elem.name == "C"
                    && elem
                        .attributes
                        .get("Type")
                        .map(|t| t == "Page")
                        .unwrap_or(false)
                {
                    return elem.attributes.get("U").map(|u| format!("uuid:{}", u));
                }
                for child in &elem.children {
                    if let Some(e) = child.as_element()
                        && let Some(r) = find_page(e)
                    {
                        return Some(r);
                    }
                }
                None
            }
            find_page(&self.root)
        } else {
            None
        }
    }
}
