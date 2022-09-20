#[derive(Debug)]
#[allow(dead_code)]
pub struct SongEntry {
    pub album_track: i16,           // short YEP
    pub chart_name: String,         // string YEP
    pub charts: i64,                // GStruct6, change later YEP
    pub checksum: [u8; 16],         // SongHash YEP
    //containers: String,         // dict<string, GClass9> PRIVATE, change later
    pub date_added: i64,            // DateTime, change later YEP
    //filtered: bool,             // bool
    pub folder_path: String,        // string YEP
    pub force_five_lane: bool,      // bool YEP
    pub force_pro_drums: bool,      // bool YEP
    pub icon_name: String,          // string YEP
    pub intensities: [i8; 10],      // sbyte[] PRIVATE YEP
    //is_available_online: bool,  // bool
    pub is_enc: bool,               // bool YEP
    //is_midi_chart_cache: bool,  // bool PRIVATE
    //is_type_cached: bool,       // bool PRIVATE
    pub lyrics: bool,               // bool YEP
    pub metadata: [String; 7],      // GClass47[] PRIVATE YEP
    //metadata_cache: String,     // string[] PRIVATE
    //metadata_loaded: bool,      // bool
    pub modchart: bool,             // bool YEP
    pub playlist_track: i16,        // short YEP
    pub preview_start: i32,         // int YEP
    //scores: String,             // GClass55, Change later
    //song_enc: String,           // GClass9, change later
    pub song_length: i32,           // int YEP
    pub sub_playlist: String,       // string YEP
    pub top_level_playlist: String, // string YEP
    pub video_background: bool,     // bool YEP
}