use spring::ResultExt;

const EFFECT_VOLUME: f32 = 40.0;

pub fn play_ui(spring: &spring::Spring<'_, spring::Ui>, file: &str) {
    spring
        .sound()
        .play_sound_file(
            file,
            EFFECT_VOLUME,
            spring::Float3::default(),
            spring::Float3::default(),
            0,
        )
        .log_err_once(spring, "Area 17 UI sound");
}
