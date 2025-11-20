use std::{ env, f32::consts::PI, fs::{self, File, read_to_string}, i16, io::BufWriter, path::Path};

use hound::WavWriter;
use rand::{Rng, rng, rngs::ThreadRng};
const CONFIG_PATH_UNIX:&'static str="/.config/mlg/config.txt";
const CONFIG_PATH_WINDOWS:&'static str="\\Roaming\\mlg\\config.txt";
#[derive(Clone, Copy)]
struct Note {
    frequency:f32,
    beats:f32,
}
impl Note {

    const C0:f32=16.35;
    const C0_SHARP:f32=17.32;
    const D0:f32=18.35;
    const D0_SHARP:f32=19.45;
    const E0:f32=20.6;
    const F0:f32=21.83;
    const F0_SHARP:f32=23.12;
    const G0:f32=24.5;
    const G0_SHARP:f32=25.96;
    const A0:f32=27.5;
    const A0_SHARP:f32=29.14;
    const B0:f32=30.87;
    const NOTES:[f32;12]=[Self::C0,Self::C0_SHARP,Self::D0,Self::D0_SHARP,Self::E0,Self::F0,Self::F0_SHARP,Self::G0,Self::G0_SHARP,Self::A0,Self::A0_SHARP,Self::B0];
    const WHOLE_ONE_BPM_SECONDS:i32=240;
    fn get_frequency(base_frequency:&f32,note_height:&u32)->f32 {
        base_frequency*2f32.powi(*note_height as i32)
    }
    fn get_speed(&self,beats_per_minute:&f32)-> f32 {
        (Self::WHOLE_ONE_BPM_SECONDS as f32*self.beats)/ beats_per_minute 
    }
    fn get_index(&self) -> u32 {
        let notes_length=Self::NOTES.len();
        for note_index in 0..notes_length {
            let note_height=(self.frequency/Self::NOTES[note_index]).log2();
            //println!("note_height:{},note_index:{}",note_height,note_index);
            if  note_height == note_height.round(){
                //print!("note_height:{}",note_height);
                return (note_height*notes_length as f32+note_index as f32) as u32;
            }
        }
        0
    }
    fn get_frequency_by_index(index:&u32) -> f32 {
        let notes_length=Self::NOTES.len();
        for note_index in 0..notes_length {
            let note_height=(index-note_index as u32) as f32/notes_length as f32;
            //println!("note_index:{},note_height:{}",note_index,note_height);
            if note_height==note_height.round() {
                return Self::get_frequency(&Self::NOTES[note_index], &(note_height as u32));
            }
        }
        0.0
    }
    fn create_note_by_index(index:&u32,beats:&f32) -> Self {
        let notes_length=Self::NOTES.len();
        for note_index in 0..notes_length {
            let note_height=(index-note_index as u32) as f32/notes_length as f32;
            //println!("note_index:{},note_height:{}",note_index,note_height);
            if note_height==note_height.round() {
                return Self::new_with_height(&Self::NOTES[note_index],&(note_height as u32),beats);
            }
        }
        Self::new_with_height(&0.0,&0, &0.0)
    }
    fn new_with_height(base_frequency:&f32,note_height:&u32,beats:&f32)->Self {
        Self { frequency: Self::get_frequency(base_frequency, note_height), beats: *beats }
    }
    fn new(frequency:&f32,beats:&f32) -> Self {
        Self {frequency:*frequency,beats:*beats}
    }
}
fn command_create(env_args:&Vec<String>) {
    let command=env_args.get(0).unwrap();
    let default_error_message=". argument missing or not valid";
    let beats=if let Some(beats)=env_args.get(1) {
        beats.parse::<f32>().expect("1. argument for the number of beats in the song should be numeric")
    } else {
        return failed_command(&command, &format!("1{}",default_error_message));
    };

    let mut arg_note_chars=env_args[2].chars();
    let frequency=if let Some(char) = arg_note_chars.nth(0) {
        match char {
            'A' => &Note::A0,
            'B' => &Note::B0,
            'C' => &Note::C0,
            'D' => &Note::D0,
            'E' => &Note::E0,
            'F' => &Note::F0,
            'G' => &Note::G0,
            _ => return failed_command(&command,"2. argument note name not valid"),
        }
    } else {
        return failed_command(&command, &format!("2{}",default_error_message));
    };
    let note_height=if let Some(note_height) = arg_note_chars.nth(0)  {
        note_height.to_digit(10).expect("2. argument note height  should be a positive integer")
    } else {
        return failed_command(&command, &format!("2{}",default_error_message))
    };
    let start_note=Note::new_with_height(frequency, &note_height, &0.0);

    let variaton_chance=if let Some(variation_chance) =env_args.get(3) {
        variation_chance.parse::<f32>().expect("3. argument for the variation chance should be numeric")
    }else {
        return failed_command(&command, &format!("3{}",default_error_message))
    };

    let note_beats:Vec<f32>=if let Some(note_beats) = env_args.get(4) {
        note_beats.split(",").map(|x| x.parse::<f32>().expect("4. argument for the beats of the notes should be a numeric array without []")).collect()
    } else {
        return failed_command(&command, &format!("4{}",default_error_message))
    };

    let bpm=if let Some(bpm) = env_args.get(5) {
        bpm.parse::<f32>().expect("5. argument for the beats per minute should be numeric")
    } else {
        return failed_command(&command, &format!("5{}",default_error_message))
    };

    let file_name=if let Some(file_name) = env_args.get(6) {
        file_name
    } else {
        return failed_command(&command, &format!("6{}",default_error_message))
    };

    create_song(&beats, &start_note, &variaton_chance, &note_beats, &bpm, file_name);
}
fn failed_command(command:&str,error_message:&str) {
    help(command);
    println!("{}",error_message)
}
fn help(command:&str) {
    let help_prompt;
    if command=="create" {
        let help_prompt_create="create [beats] [start note: [note name note height] ex. A4] [variation chance] [note beats] [beats per minute] [file name]";
        help_prompt=help_prompt_create;
    }else if command=="path" {
        let help_prompt_path="path [absolute path]";
        help_prompt=help_prompt_path;
    }else {
        let help_prompt_default="Commands:\ncreate: creates a melodie\npath: sets the output path\nhelp: gives information about commands";
        help_prompt=help_prompt_default;
    }
    println!("{}",help_prompt);
}
fn command_help(env_args:&Vec<String>) {
    if let Some(command) = env_args.get(1) {
        help(&command);
    }else {
        help("");
    }
}
fn command_path(env_args:&Vec<String>) {
    let command=env_args.get(0).unwrap();
    if let Some(new_output_path)=env_args.get(1) {
        if !Path::new(new_output_path).is_absolute() {
            return failed_command(&command, "Path has to be absolute");
        }
        let home_dir=env::home_dir().expect("No home directory found").to_str().unwrap().to_string();
        let mut add_back="";
        let config_path_string;
        if env::consts::OS=="linux" {
            config_path_string=home_dir+CONFIG_PATH_UNIX;
            if new_output_path.chars().last().unwrap()!='/' {
                add_back="/";
            }
        } else if env::consts::OS=="windows" {
            config_path_string=home_dir+CONFIG_PATH_WINDOWS;
            if new_output_path.chars().last().unwrap()!='\\' {
                add_back="\\"
            }
        } else {
            return println!("Operating system not supported");
        }
        let config_path=Path::new(&config_path_string);
        if !config_path.exists() {
            init_config(config_path);
        }
        fs::write(config_path, format!("{}{}",new_output_path,add_back)).unwrap();
    }else {
        failed_command(&command, "No path specificied");
    }
}
fn init_config(config_path:&Path) {
    fs::DirBuilder::new().create(config_path.parent().unwrap()).unwrap();
    fs::File::create(config_path).unwrap();
}
fn main() {
    let env_args:Vec<_>=env::args().skip(1).collect();
    if let Some(env_arg_zero) = env_args.get(0) {
        if env_arg_zero=="create" {
            command_create(&env_args);
        }else if env_arg_zero=="path" {
            command_path(&env_args);
        } else if env_arg_zero=="help" {
            command_help(&env_args);
        }
    }
}
fn create_song(beats:&f32,start_note:&Note,variaton_chance:&f32,note_beats:&[f32],bpm:&f32,file_name:&str) {
    let notes=create_notes(&start_note, &beats, &variaton_chance,&note_beats);
    create_tones(&notes, &bpm, &file_name);
}
fn create_tone(writer:&mut WavWriter<BufWriter<File>>,frequency:&f32,time_sec:&f32,sample_rate:&f32) {
    for t in (0..(sample_rate *time_sec) as u32).map(|x| x as f32 /sample_rate) {
        let sample=(t*frequency*2.0*PI).sin();
        let amplitude=i16::MAX as f32;
        writer.write_sample((sample * amplitude) as i16).unwrap();
    }
}
fn create_tones(notes:&[Note],bpm:&f32,file_name:&str) {
        let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let output_dir=get_output_dir(file_name);
    let mut writer = hound::WavWriter::create(output_dir, spec).unwrap();
    let sample_rate=spec.sample_rate as f32;
    for note in notes {
        create_tone(&mut writer, &note.frequency, &note.get_speed(bpm), &sample_rate);
    }
    writer.finalize().unwrap();    
}
fn get_output_dir(file_name:&str)->String {
    let home_dir=env::home_dir().expect("No home directory found").to_str().unwrap().to_string();
    let config_dir;
    if env::consts::OS=="linux" {
        config_dir=home_dir+CONFIG_PATH_UNIX;
    } else if env::consts::OS=="windos" {
        config_dir=home_dir+CONFIG_PATH_WINDOWS;
    }else {
        panic!("Operating system not supported");
    }
    
    let output_dir=read_to_string(config_dir).expect("Output path not set correctly use path to set path");
    output_dir+file_name+".wav"
}
fn create_notes(start_note:&Note,beats:&f32,variaton_chance:&f32,note_beats:&[f32])->Vec<Note> {
    let mut thread_rng=rng();
    let mut notes=Vec::<Note>::new();
    let mut beats=*beats;
    let start_note_index=start_note.get_index();
    let frequencies_base=[Note::get_frequency_by_index(&start_note_index),Note::get_frequency_by_index(&(start_note_index+5)),Note::get_frequency_by_index(&(start_note_index+7))];
    let frequencies_variation=[Note::get_frequency_by_index(&(start_note_index+1)),Note::get_frequency_by_index(&(start_note_index+6)),Note::get_frequency_by_index(&(start_note_index+8))];
    while beats>0.0 {
        let next_note=create_next_note(&frequencies_base,&frequencies_variation,variaton_chance,note_beats,&mut thread_rng);
        beats-=next_note.beats;
        notes.push(next_note);
    }
    notes
}
fn create_next_note(frequencies_base:&[f32],frequencies_variation:&[f32],variaton_chance:&f32,note_beats:&[f32],thread_rng:&mut ThreadRng)->Note {
    let note_beats_length=note_beats.len();
    let beat_index=thread_rng.random_range(0..note_beats_length);
    let beats=note_beats[beat_index];
    let frequency;
    if thread_rng.random_bool(*variaton_chance as f64) {
        let frequencies_base_length=frequencies_base.len();
        let frequency_index=thread_rng.random_range(0..frequencies_base_length);
        frequency=frequencies_base[frequency_index];
    } else {
        let frequencies_variation_length=frequencies_variation.len();
        let frequency_index=thread_rng.random_range(0..frequencies_variation_length);
        frequency=frequencies_variation[frequency_index];
    }
    let new_note=Note::new(&frequency, &beats);
    new_note
}