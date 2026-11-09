// Cyberfall: Titanfall 2 momentum slide. Crouch into a landing to slide at the speed V fell with.
// Each slide frame: ask the plugin for V's velocity, apply the difference as an impulse.

public func CyberfallCrouchHeld(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  return scriptInterface.GetActionValue(n"Crouch") > 0.0 || stateContext.GetConditionBool(n"CrouchToggled");
}

// The game only checks the slide's enter condition after a crouch input on the ground,
// so an armed landing slide switches it on itself.
@addField(PlayerPuppet)
public let m_cyberfallSlideDecisions: array<wref<SlideDecisions>>;

@wrapMethod(CrouchDecisions)
protected func OnAttach(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(stateContext, scriptInterface);
  let slide: ref<SlideDecisions> = this as SlideDecisions;
  let player: ref<PlayerPuppet> = scriptInterface.executionOwner as PlayerPuppet;
  if IsDefined(slide) && IsDefined(player) {
    let live: array<wref<SlideDecisions>>;
    for decisions in player.m_cyberfallSlideDecisions {
      if IsDefined(decisions) {
        ArrayPush(live, decisions);
      };
    };
    ArrayPush(live, slide);
    player.m_cyberfallSlideDecisions = live;
  };
}

// Every air frame: fall speed and crouch input. With toggle crouch, a tap or hold toggles.
public func CyberfallSlideNoteAirFrame(timeDelta: Float, stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  let velocity: Vector4 = DefaultTransition.GetLinearVelocity(scriptInterface);
  CyberfallSlideNoteAir(velocity.X, velocity.Y,
    scriptInterface.IsActionJustPressed(n"Crouch"),
    CyberfallCrouchHeld(stateContext, scriptInterface),
    scriptInterface.IsActionJustTapped(n"ToggleCrouch") || scriptInterface.IsActionJustHeld(n"ToggleCrouch"),
    timeDelta);
  let player: ref<PlayerPuppet> = scriptInterface.executionOwner as PlayerPuppet;
  if CyberfallSlideWantsLanding() && IsDefined(player) {
    for decisions in player.m_cyberfallSlideDecisions {
      if IsDefined(decisions) {
        decisions.EnableOnEnterCondition(true);
      };
    };
  };
}

public func CyberfallSlideOnLanding(grounded: Bool) -> Bool {
  return grounded && CyberfallSlideEnabled() && CyberfallSlideWantsLanding();
}

// Landing from a sprint goes straight into the slide.
@wrapMethod(SlideDecisions)
protected const func EnterCondition(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallSlideOnLanding(this.IsTouchingGround(scriptInterface)) {
    return true;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

// So does landing to a stand.
@wrapMethod(StandDecisions)
protected final const func ToSlide(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallSlideOnLanding(this.IsTouchingGround(scriptInterface)) {
    return true;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

// The plugin ended the slide: leave now, not after the minimum slide time.
@wrapMethod(SlideDecisions)
protected const func ShouldExit(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallSlideEnabled() && !CyberfallSlideActive() {
    return true;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

// Sprint speed for the slide, and a landing that didn't slide forgets the fall.
@wrapMethod(LocomotionEventsTransition)
public func OnUpdate(timeDelta: Float, stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(timeDelta, stateContext, scriptInterface);
  if Equals(stateContext.GetStateMachineCurrentState(n"Locomotion"), n"sprint") {
    CyberfallSlideSetSprintSpeed(scriptInterface.GetStatsSystem().GetStatValue(Cast<StatsObjectID>(scriptInterface.ownerEntityID), gamedataStatType.MaxSpeed));
  };
  if CyberfallSlideWantsLanding() && this.GetInStateTime() > 0.1 && this.IsTouchingGround(scriptInterface) {
    CyberfallSlideLanded();
  };
}

@wrapMethod(SlideEvents)
public func OnEnter(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(stateContext, scriptInterface);
  if !CyberfallSlideEnabled() {
    return;
  };
  let fromLanding: Bool = CyberfallSlideWantsLanding();
  let velocity: Vector4 = DefaultTransition.GetLinearVelocity(scriptInterface);
  let now: Float = EngineTime.ToFloat(GameInstance.GetSimTime(scriptInterface.owner.GetGame()));
  CyberfallSlideBegin(velocity.X, velocity.Y, now, fromLanding);
  // A tap in the air doesn't toggle crouch on, so landing into the slide does.
  if fromLanding && scriptInterface.GetActionValue(n"Crouch") <= 0.0 {
    stateContext.SetConditionBoolParameter(n"CrouchToggled", true, true);
  };
}

@wrapMethod(SlideEvents)
protected func OnUpdate(timeDelta: Float, stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(timeDelta, stateContext, scriptInterface);
  if !CyberfallSlideEnabled() || !CyberfallSlideActive() {
    return;
  };
  let velocity: Vector4 = DefaultTransition.GetLinearVelocity(scriptInterface);
  let facing: Vector4 = scriptInterface.executionOwner.GetWorldForward();
  let held: Bool = CyberfallCrouchHeld(stateContext, scriptInterface);
  CyberfallSlideSetInput(scriptInterface.GetActionValue(n"MoveY"), scriptInterface.GetActionValue(n"MoveX"), facing.X, facing.Y, held);
  let ground: Vector4 = this.CyberfallGroundNormal(scriptInterface);
  let going: Bool = CyberfallSlideStep(velocity.X, velocity.Y, velocity.Z, ground.X, ground.Y, ground.Z, timeDelta);
  let wanted: Vector4 = new Vector4(CyberfallSlideVelocityX(), CyberfallSlideVelocityY(), CyberfallSlideVelocityZ(), 0.0);
  stateContext.SetTemporaryVectorParameter(n"impulse", wanted - velocity, true);
  // Kerenzikov keeps a slow slide going.
  let kerenzikov: Bool = Equals(stateContext.GetStateMachineCurrentState(n"TimeDilation"), n"kerenzikov");
  if !going && !(kerenzikov && held) {
    CyberfallSlideEnd();
  };
}

@wrapMethod(SlideEvents)
public func OnExit(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  CyberfallSlideEnd();
  wrappedMethod(stateContext, scriptInterface);
}

@wrapMethod(SlideEvents)
public func OnForcedExit(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  CyberfallSlideEnd();
  wrappedMethod(stateContext, scriptInterface);
}

// Ground normal below V, or zero.
@addMethod(SlideEvents)
private final func CyberfallGroundNormal(scriptInterface: ref<StateGameScriptInterface>) -> Vector4 {
  let hit: TraceResult;
  let position: Vector4 = DefaultTransition.GetPlayerPosition(scriptInterface);
  let from: Vector4 = position + new Vector4(0.0, 0.0, 0.1, 0.0);
  let to: Vector4 = position - new Vector4(0.0, 0.0, 0.5, 0.0);
  if GameInstance.GetSpatialQueriesSystem(scriptInterface.owner.GetGame()).SyncRaycastByCollisionGroup(from, to, n"Static", hit, true, false) {
    return Vector4.Vector3To4(hit.normal);
  };
  return new Vector4(0.0, 0.0, 0.0, 0.0);
}
