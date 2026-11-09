// Legendary+ Reinforced Tendons fall perks, and the Air Dash perk text.

// Any Reinforced Tendons, modded variants included, at Legendary or above.
public func CyberfallHasLegendaryTendons(owner: ref<GameObject>) -> Bool {
  let player: ref<PlayerPuppet> = owner as PlayerPuppet;
  if !IsDefined(player) {
    return false;
  };
  let legs: TweakDBID = ItemID.GetTDBID(EquipmentSystem.GetData(player).GetActiveItem(gamedataEquipmentArea.LegsCW));
  if !Equals(TweakDBInterface.GetCName(legs + t".cyberwareType", n"None"), n"BoostedTendons") {
    return false;
  };
  let record: ref<Item_Record> = TweakDBInterface.GetItemRecord(legs);
  if !IsDefined(record) || !IsDefined(record.Quality()) {
    return false;
  };
  let quality: gamedataQuality = record.Quality().Type();
  return Equals(quality, gamedataQuality.Legendary)
      || Equals(quality, gamedataQuality.LegendaryPlus)
      || Equals(quality, gamedataQuality.LegendaryPlusPlus);
}

// No falling-speed limit on double jump and dash.
@wrapMethod(LocomotionTransition)
protected final const func IsCurrentFallSpeedTooFastToEnter(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  let tooFast: Bool = wrappedMethod(stateContext, scriptInterface);
  if tooFast && CyberfallWallEnabled() && CyberfallHasLegendaryTendons(scriptInterface.executionOwner) {
    return false;
  };
  return tooFast;
}

// Hard and very hard landings become regular ones. Death landings are untouched.
public func CyberfallSoftenLanding(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  let landing: Int32 = stateContext.GetIntParameter(n"LandingType", true);
  if landing != EnumInt(LandingType.Hard) && landing != EnumInt(LandingType.VeryHard) {
    return;
  };
  if CyberfallWallEnabled() && CyberfallHasLegendaryTendons(scriptInterface.executionOwner) {
    stateContext.SetPermanentIntParameter(n"LandingType", EnumInt(LandingType.Regular), true);
  };
}

public func CyberfallAirDashUnlockLevel() -> TweakDBID {
  return TweakDBInterface.GetNewPerkRecord(t"NewPerks.Reflexes_Central_Milestone_3").GetLevelsItem(2).GetID();
}

// "Unlocks the ability to Dash in midair and Wallrun.", using the line's own highlight.
public func CyberfallWithWallrun(text: String) -> String {
  let tag: String = "<Rich color=\"TooltipText.cyberwareDescriptionHighlightColor\" style=\"Semi-Bold\">";
  let start: Int32 = StrFindFirst(text, "<Rich");
  if start >= 0 {
    let rest: String = StrRight(text, StrLen(text) - start);
    let close: Int32 = StrFindFirst(rest, ">");
    if close > 0 {
      tag = StrLeft(rest, close + 1);
    };
  };
  if StrEndsWith(text, ".") {
    return StrLeft(text, StrLen(text) - 1) + " and " + tag + "Wallrun</>.";
  };
  return text + " " + tag + "Wallrun</>";
}

@wrapMethod(PerkDisplayTooltipController)
private final func GetLevelDescription(perkData: ref<BasePerkDisplayData>, levelDataRecord: ref<TweakDBRecord>) -> String {
  let text: String = wrappedMethod(perkData, levelDataRecord);
  if IsDefined(levelDataRecord) && levelDataRecord.GetID() == CyberfallAirDashUnlockLevel() {
    return CyberfallWithWallrun(text);
  };
  return text;
}
