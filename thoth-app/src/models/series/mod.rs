use serde::Deserialize;
use serde::Serialize;
use thoth_api::series::model::SeriesType;
use yew::html;
use yew::prelude::Html;
use yew::Callback;
use yew::MouseEvent;

use super::imprint::Imprint;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_id: String,
    pub series_type: SeriesType,
    pub series_name: String,
    pub issn_print: String,
    pub issn_digital: String,
    pub series_url: Option<String>,
    pub imprint: Imprint,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeDefinition {
    pub enum_values: Vec<SeriesTypeValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTypeValues {
    pub name: SeriesType,
}

impl Default for Series {
    fn default() -> Series {
        Series {
            series_id: "".to_string(),
            series_type: SeriesType::BookSeries,
            series_name: "".to_string(),
            issn_print: "".to_string(),
            issn_digital: "".to_string(),
            series_url: None,
            imprint: Default::default(),
        }
    }
}

impl Series {
    pub fn as_dropdown_item(&self, callback: Callback<MouseEvent>) -> Html {
        // since serieses dropdown has an onblur event, we need to use onmousedown instead of
        // onclick. This is not ideal, but it seems to be the only event that'd do the calback
        // without disabling onblur so that onclick can take effect
        html! {
            <div onmousedown=callback class="dropdown-item">
                { format!("{} ({}, {})", self.series_name, self.issn_print, self.issn_digital) }
            </div>
        }
    }
}

pub mod create_series_mutation;
pub mod delete_series_mutation;
pub mod series_query;
pub mod series_types_query;
pub mod serieses_query;
pub mod update_series_mutation;