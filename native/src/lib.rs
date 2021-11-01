#![warn(missing_docs)]

//! This crate contains "JS functions" for neon, allowing the user to read, merge and collate BibTex Files

// imports
use neon::prelude::*;
use std::path::PathBuf;
use crate::datatypes::bibentry::BibEntry;
use crate::string_matchers::{damerau_levenshtein, hamming, levenshtein, ngram};

// modules
mod datatypes;
mod utility;
mod parser;
mod volume_search;
mod error;
mod duplicate;
mod file_writer;
mod string_matchers;

/// Searches a given path and its sub directories for .bib files.
///
/// Given that the argument is not a JSString, an error will be thrown to the
/// javascript runtime. If the path does not exist or is improperly formatted. An error will
/// be thrown to the javascript runtime. **Note this can only be called from the JS Runtime**
pub fn search_volume(mut cx: FunctionContext) -> JsResult<JsArray> {
    let path_handle = cx.argument::<JsString>(0)?;
    let path_list: Vec<PathBuf> = volume_search::get_all_bibtex_files(path_handle.value(&mut cx))?;

    // pass into JS Array
    let js_return_array = JsArray::new(&mut cx, path_list.len() as u32);
    for (i, s) in path_list.iter().enumerate() {
        let js_string = 
            cx.string(
                s.to_str()
                .expect("Failed to convert PathBuf to String")
            );
        js_return_array.set(&mut cx, i as u32, js_string)?;
    }

    Ok(js_return_array)
}


/// Given an array of file paths, merges these into a singular .bib file - removing any duplicates
/// 
/// Will parse these files, search all entries for duplicates or near duplicates, and then interact
/// with the user to decide how to handle all of these. **Note this can only be called from the JS Runtime**
pub fn merge_bibtex_files(mut cx: FunctionContext) -> JsResult<JsObject> {
    let path_list_js_array = cx.argument::<JsArray>(0)?;
    let js_output_file_path = cx.argument::<JsString>(1)?;
    
    let path_list = utility::js_string_array_to_vec(path_list_js_array, &mut cx)?;
    let output_file_path = js_output_file_path.value(&mut cx);
    // if path_list.len() <= 1 {
    //     // TODO: create another function for performing same duplication tests on single string
    //     return error::create_error_object(String::from("Only 1 File Submitted"), &mut cx)
    // }
    if !utility::is_files_all_valid(&path_list) {
        return error::create_error_object(String::from("Invalid File Type Found"), &mut cx)
    }

    let file_contents = utility::read_files_into_strings(path_list)?;

    let mut entries: Vec<BibEntry> = Vec::new();
    for file_content in file_contents {
        let mut temp = parser::bibtex_parser::parse_bibtex_string(file_content)?;
        entries.append(&mut temp);
    }

    entries = duplicate::remove_direct_duplicates(entries)?;
    // TODO: check if is empty to prevent writing empty file - these checks can also be done frontend

    file_writer::write_entries_to_file(entries, output_file_path)?;

    utility::create_success_return_object(&mut cx)
}

/// Given a .bib file path will parse it, and return the entries as a JS array of objects.
/// 
/// Given a .bib path it will parse the file, throwing an error if any sanitisation errors are found.
/// It will return the JS object from a Rust Struct, which for each entry has the following format:
/// ```
/// pub struct BibEntry {
///    pub entry_type: String,
///    pub name: String,
///    pub fields: Vec<Field>,
///}
/// ```
pub fn parse_bibtex_file(mut cx: FunctionContext) -> JsResult<JsObject> {
    // TODO: check all paths are .bib files
    let path_arg = cx.argument::<JsString>(0)?;
    let path = path_arg.value(&mut cx);

    if !utility::is_file_valid(&path) {
        return error::create_error_object(String::from("File Not a bibtex file"), &mut cx);
    }

    let file_content = utility::read_file(path)?;
    let entries = parser::bibtex_parser::parse_bibtex_string(file_content)?;

    utility::create_entries_return_object(entries, &mut cx)
}

/// Given two strings, will compute the hamming distance between them
/// 
/// Given two strings, using the hamming distance algorithm to work out the distance between them
/// Returns the distance as a javascript number.
pub fn hamming(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let string1_handle = cx.argument::<JsString>(0)?;
    let string2_handle = cx.argument::<JsString>(1)?;
    
    let string1 = string1_handle.value(&mut cx);
    let string2 = string2_handle.value(&mut cx);

    println!("String 1: {}\nString 2: {}", &string1, &string2);
    let result = hamming::compute(&string1, &string2);

    Ok(cx.number(result))
}

/// Given two strings, computes the levenshtein difference between them
/// 
/// Given two strings, uses the levenshtein algorithm to work out the distance between them
/// This algorithm checks for the minimum number of insertions, deletions or subtitutions required to change one string into the other
pub fn levenshtein(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let string1_handle = cx.argument::<JsString>(0)?;
    let string2_handle = cx.argument::<JsString>(1)?;
    
    let string1 = string1_handle.value(&mut cx);
    let string2 = string2_handle.value(&mut cx);

    println!("String 1: {}\nString 2: {}", &string1, &string2);
    let result = levenshtein::compute(&string1, &string2);

    Ok(cx.number(result))
}

/// Given two strings, computes the damerau levenshtein difference between them
/// 
/// Given two strings, uses the damerau levenshtein algorithm to work out the distance between them
/// Checks same operations as levenshtein (isnertions, deletions or substitutions), but also includes transposition of two adjacent characters
/// Note this is the simpler "Optimal string alignment distance" algorithm, which is slightly simpler than the "true" demerau levenshtein distance
/// The difference between the two algorithms can be seen here https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
pub fn damerau_levenshtein(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let string1_handle = cx.argument::<JsString>(0)?;
    let string2_handle = cx.argument::<JsString>(1)?;
    
    let string1 = string1_handle.value(&mut cx);
    let string2 = string2_handle.value(&mut cx);

    println!("String 1: {}\nString 2: {}", &string1, &string2);
    let result = damerau_levenshtein::compute(&string1, &string2);

    Ok(cx.number(result))
}

pub fn ngram(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let string1_handle = cx.argument::<JsString>(0)?;
    let string2_handle = cx.argument::<JsString>(1)?;
    let ngram_handle = cx.argument::<JsNumber>(2)?;
    
    let string1 = string1_handle.value(&mut cx);
    let string2 = string2_handle.value(&mut cx);
    let n = ngram_handle.value(&mut cx) as i32;

    println!("String 1: {}\nString 2: {}", &string1, &string2);
    let result = ngram::compute(&string1, &string2, n);

    Ok(cx.number(result))
}


#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("searchVolume", search_volume)?;
    cx.export_function("mergeBibTexFiles", merge_bibtex_files)?;
    cx.export_function("parseBibTexFile", parse_bibtex_file)?;
    cx.export_function("hamming", hamming)?;
    cx.export_function("levenshtein", levenshtein)?;
    cx.export_function("damerauLevenshtein", damerau_levenshtein)?;
    cx.export_function("ngram", ngram)?;
    Ok(())
}
