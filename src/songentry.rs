use serde::Serialize;

const EMPTY_STRING: String = String::new();

#[derive(Serialize, Debug)]
pub struct SongEntry {

    // normal format
    pub album_track: i16,           // short
    pub chart_name: String,         // string
    pub charts: i64,                // GStruct6
    pub checksum: [u8; 16],         // SongHash
    pub date_added: i64,            // DateTime
    pub folder_path: String,        // string
    pub force_five_lane: bool,      // bool
    pub force_pro_drums: bool,      // bool
    pub icon_name: String,          // string
    pub intensities: [i8; 10],      // sbyte[]
    pub is_enc: bool,               // bool
    pub lyrics: bool,               // bool
    pub metadata: [String; 7],      // GClass47[]
    pub modchart: bool,             // bool
    pub playlist_track: i16,        // short
    pub preview_start: i32,         // int
    pub song_length: i32,           // int
    pub sub_playlist: String,       // string
    pub top_level_playlist: String, // string
    pub video_background: bool,     // bool

    // unused stuff from internal script
    //containers: String,           // dict<string, GClass9> PRIVATE
    //filtered: bool,               // bool
    //is_available_online: bool,    // bool
    //is_midi_chart_cache: bool,    // bool PRIVATE
    //is_type_cached: bool,         // bool PRIVATE
    //metadata_cache: String,       // string[] PRIVATE
    //metadata_loaded: bool,        // bool
    //scores: String,               // GClass55
    //song_enc: String,             // GClass9
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
