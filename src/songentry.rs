use serde::Serialize;

const EMPTY_STRING: String = String::new();

#[derive(Serialize, Debug)]
#[allow(dead_code)]
pub struct SongEntry {
    pub album_track: i16,           // short YEP
    pub chart_name: String,         // string YEP2
    pub charts: i64,                // GStruct6 change
    pub checksum: [u8; 16],         // SongHash change
    pub date_added: i64,            // DateTime change
    pub folder_path: String,        // string
    pub force_five_lane: bool,      // bool YEP
    pub force_pro_drums: bool,      // bool YEP
    pub icon_name: String,          // string YEP
    pub intensities: [i8; 10],      // sbyte[] YEP
    pub is_enc: bool,               // bool
    pub lyrics: bool,               // bool
    pub metadata: [String; 7],      // GClass47[] change YEP
    pub modchart: bool,             // bool YEP
    pub playlist_track: i16,        // short YEP
    pub preview_start: i32,         // int YEP
    pub song_length: i32,           // int YEP
    pub sub_playlist: String,       // string YEP
    pub top_level_playlist: String, // string YEP
    pub video_background: bool,     // bool YEP2

    //containers: String,           // dict<string, GClass9> PRIVATE, change later
    //filtered: bool,               // bool
    //is_available_online: bool,    // bool
    //is_midi_chart_cache: bool,    // bool PRIVATE
    //is_type_cached: bool,         // bool PRIVATE
    //metadata_cache: String,       // string[] PRIVATE
    //metadata_loaded: bool,        // bool
    //scores: String,               // GClass55, Change later
    //song_enc: String,             // GClass9, change later
}
impl SongEntry {
    pub fn default() -> SongEntry {
        SongEntry {
            album_track: 16000,
            chart_name: EMPTY_STRING,
            charts: 0,
            checksum: [0; 16],
            date_added: 0,
            folder_path: EMPTY_STRING,
            force_five_lane: false,
            force_pro_drums: false,
            icon_name: EMPTY_STRING,
            intensities: [-1; 10],
            is_enc: false,
            lyrics: false,
            metadata: [EMPTY_STRING; 7],
            modchart: false,
            playlist_track: 16000,
            preview_start: -1,
            song_length: 0,
            sub_playlist: EMPTY_STRING,
            top_level_playlist: EMPTY_STRING,
            video_background: false,
        }
    }
}
