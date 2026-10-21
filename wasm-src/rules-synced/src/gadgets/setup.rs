use spring_addons::register_gadgets;
use spring_addons::rules::GadgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(
    handler: &mut GadgetHandler<Global>,
    _spring: &spring::Spring<'_, spring::RulesSynced>,
) {
    register_gadgets!(handler, [GameLoadGadget::new()]);

    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [DbgIntegrationGadget::new()]);
    #[cfg(feature = "e2e")]
    // The technology-panel fixture also uses e2e, without combat input.
    if _spring
        .game()
        .get_mod_option("visual_test")
        .is_ok_and(|option| option.exists && option.value == "1")
    {
        register_gadgets!(handler, [DbgVisualIntegrationGadget::new()]);
    }

    register_gadgets!(
        handler,
        [
            UnitFallDamageGadget::new(),
            UnitUfoControllerGadget::new(),
            UnitCivilianSpawnerGadget::new(),
            UnitDronesGadget::new(),
            UnitUfoAbductionGadget::new(),
            WeaponHandlersGadget::new(),
            UpgradesGadget::new(),
            GameTerrainGadget::new(),
            UnitSetupGadget::new(),
            UnitScriptsGadget::new(),
            ApiStatusGadget::new(),
            UnitBuildingsGadget::new(),
        ]
    );

    #[cfg(feature = "diagnostics")]
    register_gadgets!(handler, [DbgDiagnosticsGadget::new()]);
}
