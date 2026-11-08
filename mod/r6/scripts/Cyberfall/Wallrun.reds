// Cyberfall: Titanfall 2 wallrunning.
// Each air frame: read the wall, ask the plugin for V's velocity, apply the difference as an impulse.

// Air states a wallrun can happen in.
public func CyberfallIsWallrunAirState(state: CName) -> Bool {
  return Equals(state, n"jump") || Equals(state, n"doubleJump") || Equals(state, n"dodgeAir")
      || Equals(state, n"fall") || Equals(state, n"unsecureFootingFall") || Equals(state, n"slideFall")
      || Equals(state, n"chargeJump");
}

// Air Dash, Reflexes tree.
public func CyberfallHasAirDash(scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  let owner: ref<GameObject> = scriptInterface.executionOwner;
  return PlayerDevelopmentSystem.GetInstance(owner).IsNewPerkBought(owner, gamedataNewPerkType.Reflexes_Central_Milestone_3) == 3;
}

public func CyberfallMayWallrun(scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if !CyberfallWallEnabled() {
    return false;
  };
  return !CyberfallWallNeedsPerk() || CyberfallHasAirDash(scriptInterface);
}

@addField(LocomotionAirEvents)
private let m_cyberfallGravityOff: Bool;

@wrapMethod(LocomotionAirEvents)
protected func OnUpdate(timeDelta: Float, stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(timeDelta, stateContext, scriptInterface);
  CyberfallSoftenLanding(stateContext, scriptInterface);
  this.CyberfallWallUpdate(timeDelta, stateContext, scriptInterface);
}

@wrapMethod(LocomotionAirEvents)
public func OnExit(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  this.CyberfallRestoreGravity(stateContext, scriptInterface);
  wrappedMethod(stateContext, scriptInterface);
}

// Jump on a wall, or just after leaving one, is a kick, not a double jump.
@wrapMethod(DoubleJumpDecisions)
protected const func EnterCondition(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallWallIsRunning() || CyberfallWallEnded() == 1 || CyberfallWallKickWindow() {
    return false;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

// The air dash ("dodge") is a ground state class, so this also fires mid-air.
@wrapMethod(LocomotionGroundEvents)
public func OnEnter(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  wrappedMethod(stateContext, scriptInterface);
  if this.IsTouchingGround(scriptInterface) {
    CyberfallWallTouchedGround();
  } else {
    CyberfallWallAirDash();
  };
  CyberfallLevelCamera(scriptInterface);
}

// unsecureFootingFall (slipping off an edge) can't double jump or dash until landing,
// and the game also picks it mid-wallrun. Fall normally instead.
@wrapMethod(UnsecureFootingFallDecisions)
protected const func EnterCondition(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallWallEnabled() {
    return false;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

// doubleJump can't re-enter itself, so once a wall gives the double jump back, drop to fall.
@wrapMethod(FallDecisions)
protected const func EnterCondition(const stateContext: ref<StateContext>, const scriptInterface: ref<StateGameScriptInterface>) -> Bool {
  if CyberfallWallEnabled() && !CyberfallWallIsRunning()
      && Equals(stateContext.GetStateMachineCurrentState(n"Locomotion"), n"doubleJump")
      && stateContext.GetIntParameter(n"currentNumberOfJumps", true) == 0 {
    return true;
  };
  return wrappedMethod(stateContext, scriptInterface);
}

public func CyberfallRollQuat(roll: Float) -> Quaternion {
  let angles: EulerAngles;
  angles.Roll = roll;
  return EulerAngles.ToQuat(angles);
}

public func CyberfallLevelCamera(scriptInterface: ref<StateGameScriptInterface>) -> Void {
  let player: ref<PlayerPuppet> = scriptInterface.executionOwner as PlayerPuppet;
  if IsDefined(player) {
    CyberfallWallCameraRoll(1000.0);
    player.GetFPPCameraComponent().SetLocalOrientation(CyberfallRollQuat(0.0));
  };
}

@addMethod(LocomotionAirEvents)
private final func CyberfallWallUpdate(timeDelta: Float, stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  let state: CName = stateContext.GetStateMachineCurrentState(n"Locomotion");
  let wasRunning: Bool = CyberfallWallIsRunning();

  if !CyberfallIsWallrunAirState(state) || this.IsTouchingGround(scriptInterface) || !CyberfallMayWallrun(scriptInterface) {
    if wasRunning {
      CyberfallWallReset();
      this.CyberfallRestoreGravity(stateContext, scriptInterface);
      CyberfallLevelCamera(scriptInterface);
    };
    return;
  };
  if wasRunning && Equals(state, n"dodgeAir") {
    CyberfallWallReset();
    this.CyberfallRestoreGravity(stateContext, scriptInterface);
    return;
  };

  let velocity: Vector4 = DefaultTransition.GetLinearVelocity(scriptInterface);
  let position: Vector4 = DefaultTransition.GetPlayerPosition(scriptInterface);
  this.CyberfallReadWall(scriptInterface, position, velocity, wasRunning);

  let facing: Vector4 = scriptInterface.executionOwner.GetWorldForward();
  CyberfallWallSetVelocity(velocity.X, velocity.Y, velocity.Z);
  CyberfallWallSetInput(
    scriptInterface.GetActionValue(n"MoveY"),
    scriptInterface.GetActionValue(n"MoveX"),
    facing.X,
    facing.Y,
    scriptInterface.IsActionJustPressed(n"Jump"),
    scriptInterface.IsActionJustPressed(n"Crouch") || scriptInterface.IsActionJustTapped(n"ToggleCrouch"));

  let running: Bool = CyberfallWallStep(this.CyberfallGroundClear(scriptInterface, position), timeDelta);
  if CyberfallWallRefreshAirMoves() {
    stateContext.SetPermanentIntParameter(n"currentNumberOfJumps", 0, true);
    stateContext.SetPermanentIntParameter(n"currentNumberOfAirDodges", 0, true);
    stateContext.SetPermanentBoolParameter(n"disableAirDash", false, true);
  };
  if !running && !wasRunning && CyberfallWallEnded() != 1 {
    CyberfallWallAdvanceAir(timeDelta);
    this.CyberfallApplyCameraRoll(scriptInterface, timeDelta);
    return;
  };

  let wanted: Vector4 = new Vector4(CyberfallWallVelocityX(), CyberfallWallVelocityY(), CyberfallWallVelocityZ(), 0.0);
  stateContext.SetTemporaryVectorParameter(n"impulse", wanted - velocity, true);
  if running {
    this.CyberfallGravityOff(stateContext);
  } else {
    this.CyberfallRestoreGravity(stateContext, scriptInterface);
  };
  this.CyberfallApplyCameraRoll(scriptInterface, timeDelta);
}

// Nearest wall at knee, chest and head height, to the sides and diagonally ahead,
// and how many of those heights see it. Distance is measured straight out from the wall.
@addMethod(LocomotionAirEvents)
private final func CyberfallReadWall(scriptInterface: ref<StateGameScriptInterface>, position: Vector4, velocity: Vector4, running: Bool) -> Void {
  let radius: Float = 0.40;
  let reach: Float = radius + CyberfallWallStickDistance();
  let queries: ref<SpatialQueriesSystem> = GameInstance.GetSpatialQueriesSystem(scriptInterface.owner.GetGame());
  let heights: array<Float>;
  let centre: Vector4;
  let heading: Vector4 = Vector4.Normalize2D(velocity);
  let side: Vector4 = new Vector4(-heading.Y, heading.X, 0.0, 0.0);
  let best: TraceResult;
  let bestDistance: Float = 1000.0;
  let hit: TraceResult;
  let directions: array<Vector4>;
  let length: Float;
  let distance: Float;
  let maxNormalZ: Float = CyberfallWallMaxNormalZ();
  let facings: array<Vector4>;
  let nearest: Float;
  let nearestFacing: Vector4;
  let seen: Int32 = 0;
  CyberfallWallSetAhead(false, 0.0, 0.0);
  if Vector4.Length2D(velocity) < 0.01 {
    CyberfallWallSetReading(false, 0.0, 0.0, 0.0, 0.0, 0);
    return;
  };
  ArrayPush(directions, side);
  ArrayPush(directions, side * -1.0);
  ArrayPush(directions, Vector4.Normalize2D(heading + side));
  ArrayPush(directions, Vector4.Normalize2D(heading - side));
  ArrayPush(heights, 0.4);
  ArrayPush(heights, 0.9);
  ArrayPush(heights, 1.4);
  for height in heights {
    centre = position + new Vector4(0.0, 0.0, height, 0.0);
    nearest = 1000.0;
    nearestFacing = new Vector4(0.0, 0.0, 0.0, 0.0);
    for direction in directions {
      length = AbsF(Vector4.Dot2D(direction, heading)) > 0.1 ? reach * 1.42 : reach;
      if queries.SyncRaycastByCollisionGroup(centre, centre + direction * length, n"Static", hit, true, false) && AbsF(hit.normal.Z) < maxNormalZ {
        distance = AbsF(Vector4.Dot2D(centre - Vector4.Vector3To4(hit.position), Vector4.Normalize2D(Vector4.Vector3To4(hit.normal)))) - radius;
        if distance < nearest {
          nearest = distance;
          nearestFacing = Vector4.Normalize2D(Vector4.Vector3To4(hit.normal));
        };
        if distance < bestDistance {
          bestDistance = distance;
          best = hit;
        };
      };
    };
    ArrayPush(facings, nearestFacing);
  };
  if bestDistance >= 1000.0 {
    CyberfallWallSetReading(false, 0.0, 0.0, 0.0, 0.0, 0);
    return;
  };
  for facing in facings {
    if Vector4.Dot2D(facing, Vector4.Normalize2D(Vector4.Vector3To4(best.normal))) > 0.9 {
      seen += 1;
    };
  };
  CyberfallWallSetReading(true, best.normal.X, best.normal.Y, best.normal.Z, MaxF(bestDistance, 0.0), seen);
  if running {
    this.CyberfallProbeAhead(queries, position, velocity, best, bestDistance + radius, heights, radius);
  };
}

// Anything sticking out of the wall ahead, and how far out V has to go to get around it.
@addMethod(LocomotionAirEvents)
private final func CyberfallProbeAhead(queries: ref<SpatialQueriesSystem>, position: Vector4, velocity: Vector4, wall: TraceResult, centreToWall: Float, heights: array<Float>, radius: Float) -> Void {
  let away: Vector4 = Vector4.Normalize2D(Vector4.Vector3To4(wall.normal));
  let along: Vector4 = velocity - away * Vector4.Dot2D(velocity, away);
  let tangent: Vector4 = Vector4.Normalize2D(along);
  let look: Float = radius + Vector4.Length2D(along) * CyberfallWallBumpLookahead();
  let wallPoint: Vector4 = position - away * centreToWall;
  let blocked: Float = this.CyberfallFirstBlock(queries, wallPoint, away, tangent, 0.05, look, heights);
  let clearance: Float = -1.0;
  let tries: array<Float>;
  if blocked < 0.0 {
    return;
  };
  ArrayPush(tries, 0.15);
  ArrayPush(tries, 0.3);
  ArrayPush(tries, CyberfallWallBumpMaxClearance());
  for offset in tries {
    if clearance < 0.0 && this.CyberfallFirstBlock(queries, wallPoint, away, tangent, offset, look, heights) < 0.0 {
      clearance = offset;
    };
  };
  CyberfallWallSetAhead(true, MaxF(blocked - 0.1, 0.0), clearance);
}

// Distance to the first face blocking a line `offset` out from the wall, or -1.
@addMethod(LocomotionAirEvents)
private final func CyberfallFirstBlock(queries: ref<SpatialQueriesSystem>, wallPoint: Vector4, away: Vector4, tangent: Vector4, offset: Float, look: Float, heights: array<Float>) -> Float {
  let hit: TraceResult;
  let from: Vector4;
  let distance: Float;
  let nearest: Float = -1.0;
  for height in heights {
    from = wallPoint + away * offset + new Vector4(0.0, 0.0, height, 0.0);
    if queries.SyncRaycastByCollisionGroup(from, from + tangent * look, n"Static", hit, true, false) {
      if Vector4.Dot2D(Vector4.Vector3To4(hit.normal), tangent) < -0.5 {
        distance = Vector4.Distance(from, Vector4.Vector3To4(hit.position));
        if nearest < 0.0 || distance < nearest {
          nearest = distance;
        };
      };
    };
  };
  return nearest;
}

@addMethod(LocomotionAirEvents)
private final func CyberfallGroundClear(scriptInterface: ref<StateGameScriptInterface>, position: Vector4) -> Bool {
  let hit: TraceResult;
  let from: Vector4 = position + new Vector4(0.0, 0.0, 0.1, 0.0);
  let to: Vector4 = position - new Vector4(0.0, 0.0, CyberfallWallGroundClearance(), 0.0);
  return !GameInstance.GetSpatialQueriesSystem(scriptInterface.owner.GetGame()).SyncRaycastByCollisionGroup(from, to, n"Static", hit, true, false);
}

@addMethod(LocomotionAirEvents)
private final func CyberfallGravityOff(stateContext: ref<StateContext>) -> Void {
  let params: ref<LocomotionParameters> = new LocomotionParameters();
  this.GetStateDefaultLocomotionParameters(params);
  params.SetUpwardsGravity(0.0);
  params.SetDownwardsGravity(0.0);
  stateContext.SetTemporaryScriptableParameter(n"locomotionParameters", params, true);
  this.m_cyberfallGravityOff = true;
}

@addMethod(LocomotionAirEvents)
private final func CyberfallRestoreGravity(stateContext: ref<StateContext>, scriptInterface: ref<StateGameScriptInterface>) -> Void {
  if this.m_cyberfallGravityOff {
    this.SetLocomotionParameters(stateContext, scriptInterface);
    this.m_cyberfallGravityOff = false;
  };
}

@addMethod(LocomotionAirEvents)
private final func CyberfallApplyCameraRoll(scriptInterface: ref<StateGameScriptInterface>, timeDelta: Float) -> Void {
  let player: ref<PlayerPuppet> = scriptInterface.executionOwner as PlayerPuppet;
  if !IsDefined(player) {
    return;
  };
  let roll: Float = Rad2Deg(CyberfallWallCameraRoll(timeDelta));
  player.GetFPPCameraComponent().SetLocalOrientation(CyberfallRollQuat(roll));
}
