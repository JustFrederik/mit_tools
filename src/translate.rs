use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use model_manager::model_manager::ModelManager;
use translators::detector::Detectors;
use translators::languages::Language;
use translators::model_register::register;
use translators::translators::{Translator, Translators};
use translators::translators::chainer::{TranslatorInfo, TranslatorSelectorInfo};
use translators::translators::context::Context;
use translators::translators::offline::ctranslate2::model_management::{CTranslateModels, ModelLifetime, TokenizerModels};
use translators::translators::tokens::Tokens;
use pyo3::prelude::*;

#[pyclass(unsendable)]
pub struct Data {
    mm: ModelManager,
    tokenizer_models: TokenizerModels,
    ctranslate_models: CTranslateModels,
    context: Vec<Context>,
    retry_delay: Option<Duration>, retry_count: Option<u32>,
    tokens: Tokens,
    selector: TranslatorSelectorInfo
}

impl Data {
    pub fn new(chat_gpt_context: Option<String>) -> Result<Self, String> {
        let mut model_manager = ModelManager::new().map_err(|_| "Failed to create model manager".to_string())?;
        register(&mut model_manager);
        let tk = TokenizerModels::new(ModelLifetime::KeepAlive);
        let tt = CTranslateModels::new(ModelLifetime::KeepAlive);
        let mut ctx = vec![];
        if let Some(c) = chat_gpt_context {
            ctx.push(Context::ChatGPT(c));
        }
        let selector = TranslatorSelectorInfo::List(vec![TranslatorInfo { translator: Translator::Google, to: Language::English }]);
        Ok(Self { mm: model_manager, tokenizer_models: tk, ctranslate_models: tt, context: ctx, retry_delay: None, retry_count: None, tokens: Tokens::empty(), selector })
    }

    pub fn generate_selector_selective(&mut self, target: &str, default_translator: &str, translators: Vec<(&str, &str)>) -> Result<(), String>{
        let mut hashmap = HashMap::new();
        for (lang, translator) in translators {
            hashmap.insert(Language::from_str(lang).map_err(|_|"Language does not exist".to_string())?, Translator::from_str(translator).map_err(|_|"Translator format wrong".to_string())?);
        }
        self.selector = TranslatorSelectorInfo::Selective(hashmap, TranslatorInfo {
            translator: Translator::from_str(default_translator).map_err(|_|"Translator format wrong".to_string())?,
            to: Language::from_str(target).map_err(|_|"Language does not exist".to_string())?,
        });
        Ok(())
    }

    pub fn generate_chain(&mut self, translators: Vec<(&str, &str)>) -> Result<(), String>{
        let mut v = vec![];
        for (translators, lang) in translators {
            v.push(TranslatorInfo {
                translator: Translator::from_str(translators).map_err(|_|"Translator format wrong".to_string())?,
                to: Language::from_str(lang).map_err(|_|"Language does not exist".to_string())?
            })
        }
        self.selector = TranslatorSelectorInfo::Chain(v);
        Ok(())
    }

    pub fn generate_selective_chain(&mut self, translators: Vec<(&str, &str, &str)>, default_target: &str, default_translator: &str) -> Result<(), String>{
        let mut v = HashMap::new();
        for (from, translators, to) in translators {
            v.insert(Language::from_str(from).map_err(|_|"Language does not exist".to_string())?, TranslatorInfo {
                translator: Translator::from_str(translators).map_err(|_|"Translator format wrong".to_string())?,
                to: Language::from_str(to).map_err(|_|"Language does not exist".to_string())?
            });
        }
        self.selector = TranslatorSelectorInfo::SelectiveChain(v, TranslatorInfo {
            translator: Translator::from_str(default_translator).map_err(|_|"Translator format wrong".to_string())?,
            to: Language::from_str(default_target).map_err(|_|"Language does not exist".to_string())?,
        });
        Ok(())

    }

    pub fn generate_list(&mut self, translators: Vec<(&str, &str)>) -> Result<(), String>{
        let mut v = vec![];
        for (translators, lang) in translators {
            v.push(TranslatorInfo {
                translator: Translator::from_str(translators).map_err(|_|"Translator format wrong".to_string())?,
                to: Language::from_str(lang).map_err(|_|"Language does not exist".to_string())?
            })
        }
        self.selector = TranslatorSelectorInfo::List(v);
        Ok(())
    }

    pub fn get_new_translator_instance(&self) -> Result<Translate, String>{
        Translate::new(self.retry_delay, self.retry_count, self.selector.clone(), &self.mm, self.tokens.clone())
    }
}

#[pyclass(unsendable)]
pub struct Translate {
    translators: Translators,
}

impl Translate {
    pub fn new(retry_delay: Option<Duration>, retry_count: Option<u32>, selector: TranslatorSelectorInfo, mm: &ModelManager, tokens: Tokens) -> Result<Self, String> {
        let temp = Translators::new(
            Some(tokens),
            selector,
            retry_delay,
            retry_count,
            Detectors::Whatlang,
            mm,
        ).map_err(|_| "Failed to create translators".to_string())?;
        Ok(Self { translators: temp })
    }

    pub fn translate(&self, text: String, data: &mut Data) -> Result<String, String> {
        let v = self.translators.translate(text, None, &data.context, &mut data.ctranslate_models, &mut data.tokenizer_models).map_err(|_| "Failed to translate".to_string())?;
        v.last().ok_or("Failed to get translation".to_string()).map(|v| v.text.to_string())
    }

    pub fn translate_vec(&self, text: Vec<String>, data: &mut Data) -> Result<Vec<String>, String> {
        let v = self.translators.translate_vec(text, None, &data.context, &mut data.ctranslate_models, &mut data.tokenizer_models).map_err(|_| "Failed to translate".to_string())?;
        v.last().ok_or("Failed to get translation".to_string()).map(|v| v.text.clone())
    }
}