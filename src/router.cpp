#include <algorithm>
#include <chrono>

#include <fmt/format.h>

#include "router.h"
#include "ucs.h"

Router::Router(
    Layout& _layout, ConnectionIdxVec& connectionIdxVec, ThreadStop& threadStop,
    Layout& _inputLayout, Layout& _currentLayout,
    const TimeDuration& _maxRenderDelay)
  : layout_(_layout),
    connectionIdxVec_(connectionIdxVec),
    inputLayout_(_inputLayout),
    currentLayout_(_currentLayout),
    nets_(_layout),
    threadStop_(threadStop),
    allPinSet_(ViaSet()),
    maxRenderDelay_(_maxRenderDelay)
{
  viaTraceVec_ = WireLayerViaVec(layout_.gridW * layout_.gridH);
}

bool Router::route()
{
  blockComponentFootprints();
  joinAllConnections();
  registerActiveComponentPins();
  auto isAborted = routeAll();
  layout_.stripCutVec = findStripCuts();
  layout_.cost +=
      layout_.settings.cut_cost * static_cast<int>(layout_.stripCutVec.size());
  if (layout_.hasError) {
    layout_.diagTraceVec = viaTraceVec_;
  }
  return isAborted;
}

//
// Private
//

bool Router::routeAll()
{
  bool isAborted = false;
  auto startTime = std::chrono::steady_clock::now();
  auto connectionViaVec = layout_.circuit.genConnectionViaVec();
  layout_.routeStatusVec.resize(connectionViaVec.size(), false);
  for (auto connectionIdx : connectionIdxVec_) {
    auto viaStartEnd = connectionViaVec[connectionIdx];
    auto routeWasFound = findCompleteRoute(viaStartEnd);
    layout_.routeStatusVec[connectionIdx] = routeWasFound;

#ifndef NDEBUG
// For debugging, exit router after a given number of routes.
//  static int breakIdx = 0;
//    if (++breakIdx == 3) {
//      layout_.hasError = true;
//      layout_.errorStringVec.push_back(fmt::format("Forced exit after route
//      {}", breakIdx));
//    }
#endif

    if (threadStop_.isStopped()) {
      isAborted = true;
      break;
    }
    {
      auto lock = inputLayout_.scopeLock();
      if (!layout_.isBasedOn(inputLayout_)) {
        isAborted = true;
        break;
      }
    }
    if (layout_.hasError) {
      break;
    }
    if (std::chrono::steady_clock::now() - startTime > maxRenderDelay_) {
      auto lock = currentLayout_.scopeLock();
      currentLayout_ = layout_;
      startTime = std::chrono::steady_clock::now();
    }
  }
  return isAborted;
}

// There are two main approaches possible when routing with potential shortcut.
//
// (1) If, when routing from A to B, the router starts at A, finds a shortcut to
// B, and routes only to the shortcut, B remains unconnected. It then becomes
// necessary to do a second route, starting at B, and routing to A or a shortcut
// to A.
//
// (2) However, if Uniform Cost Search is instead allowed to flow through
// shortcuts but does not stop there, one gets a route that always connects A
// and B, but will follow low cost routes along shortcuts when possible.
//
// I've currently implemented (2). I'm not sure if (1) would create any
// different routes.

bool Router::findCompleteRoute(const StartEndVia& viaStartEnd)
{
  auto routeWasFound = findRoute(viaStartEnd);
  if (routeWasFound) {
    ++layout_.nCompletedRoutes;
  }
  else {
    ++layout_.nFailedRoutes;
  }
  return routeWasFound;
}

bool Router::findRoute(const StartEndVia& viaStartEnd)
{
  UniformCostSearch ucs(*this, layout_, nets_, viaStartEnd);
  auto routeStepVec = ucs.findLowestCostRoute();
  if (layout_.hasError || !routeStepVec.size()) {
    return false;
  }
  blockRoute(routeStepVec);
  nets_.connectRoute(routeStepVec);
  auto routeSectionVec = condenseRoute(routeStepVec);
  addWireJumps(routeSectionVec);
  layout_.routeVec.push_back(routeSectionVec);
  return true;
}

// - Route always starts and ends on wire layer.
// - Through to wire always starts a wire section.
// - Through to strip always ends a wire section.
// - Everything else is a strip section.
RouteSectionVec Router::condenseRoute(const RouteStepVec& routeStepVec)
{
  RouteSectionVec routeSectionVec;
  assert(!routeStepVec.begin()->isWireLayer);
  assert(!(routeStepVec.rend() - 1)->isWireLayer);
  auto startSection = routeStepVec.begin();
  for (auto i = routeStepVec.begin() + 1; i != routeStepVec.end(); ++i) {
    if (i->isWireLayer != (i - 1)->isWireLayer) {
      if ((i - 1) != startSection) {
        routeSectionVec.push_back(LayerStartEndVia(*startSection, *(i - 1)));
        startSection = i;
      }
    }
  }
  if (startSection != routeStepVec.end() - 1) {
    routeSectionVec.push_back(
        LayerStartEndVia(*startSection, *(routeStepVec.end() - 1)));
  }
  return routeSectionVec;
}

// Transitions
// Cuts at:
// - used <-> other used
// - used <-> other pin
// Cuts NOT at:
// - unused <-> used
// - unused <-> pin
// - used <-> same pin
StripCutVec Router::findStripCuts()
{
  StripCutVec v;
  for (int x = 0; x < layout_.gridW; ++x) {
    bool isUsed = false;
    for (int y = 1; y < layout_.gridH; ++y) {
      Via prevVia(x, y - 1);
      Via curVia(x, y);
      auto isConnected = nets_.isConnected(curVia, prevVia);
      bool isInOtherNet = nets_.hasConnection(curVia) && !isConnected;
      bool isOtherPin = isAnyPin(curVia) && !isConnected;
      if (isInOtherNet || isOtherPin) {
        if (isUsed) {
          v.push_back(curVia);
        }
        isUsed = true;
      }
    }
  }
  return v;
}

//
// Interface for Uniform Cost Search
//

bool Router::isAvailable(
    const LayerVia& via, const Via& targetVia)
{
  if (via.via.x() < 0 || via.via.y() < 0 || via.via.x() >= layout_.gridW
      || via.via.y() >= layout_.gridH) {
    return false;
  }
  if (via.isWireLayer) {
    if (isBlocked(via.via)) {
      return false;
    }
  }
  else {
    // If it has an equivalent, it must be our equivalent
    if (nets_.hasConnection(via.via) && !nets_.isConnected(via.via, targetVia)) {
      return false;
    }
    // Can go to component pin only if it's our equivalent.
    if (isAnyPin(via.via)) {
      if (!nets_.isConnected(via.via, targetVia)) {
        return false;
      }
    }
  }
  // Can go there!
  return true;
}

bool Router::isTarget(const LayerVia& via, const Via& targetVia)
{
  if (via.isWireLayer) {
    return false;
  }
  else if (isTargetPin(via, targetVia)) {
    return true;
  }
  return false;
}

bool Router::isTargetPin(const LayerVia& via, const Via& targetVia)
{
  return (via.via == targetVia).all();
}

bool Router::isAnyPin(const Via& via)
{
  return allPinSet_.count(via) > 0;
}

ValidVia& Router::wireToViaRef(const Via& via)
{
  return viaTraceVec_[layout_.idx(via)].wireToVia;
}

//
// Wire layer blocking
//

void Router::blockComponentFootprints()
{
  // Block the entire component footprint on the wire layer
  for (auto& ci : layout_.circuit.componentNameToComponentMap) {
    const auto& componentName = ci.first;
    auto footprint = layout_.circuit.calcComponentFootprint(componentName);
    for (int y = footprint.start.y(); y <= footprint.end.y(); ++y) {
      for (int x = footprint.start.x(); x <= footprint.end.x(); ++x) {
        block(Via(x, y));
      }
    }
  }
}

void Router::blockRoute(const RouteStepVec& routeStepVec)
{
  for (auto& c : routeStepVec) {
    if (c.isWireLayer) {
      block(c.via);
    }
  }
}

void Router::block(const Via& via)
{
  viaTraceVec_[layout_.idx(via)].isWireSideBlocked = true;
}

bool Router::isBlocked(const Via& via)
{
  return viaTraceVec_[layout_.idx(via)].isWireSideBlocked;
}

//
// Nets
//

void Router::joinAllConnections()
{
  for (auto& c : layout_.circuit.genConnectionViaVec()) {
    nets_.connect(c.start, c.end);
  }
}

void Router::registerActiveComponentPins()
{
  for (auto& ci : layout_.circuit.componentNameToComponentMap) {
    const auto& componentName = ci.first;
    const auto& component = ci.second;
    auto pinViaVec = layout_.circuit.calcComponentPins(componentName);
    auto pinIdx = 0;
    for (auto via : pinViaVec) {
      if (!component.dontCarePinIdxSet.count(pinIdx)) {
        allPinSet_.insert(via);
      }
      ++pinIdx;
    }
  }
}

void Router::addWireJumps(const RouteSectionVec& routeSectionVec)
{
  for (auto section : routeSectionVec) {
    const auto& start = section.start;
    const auto& end = section.end;
    assert(start.isWireLayer == end.isWireLayer);
    if (start.isWireLayer) {
      wireToViaRef(start.via) = ValidVia(end.via, true);
      wireToViaRef(end.via) = ValidVia(start.via, true);
    }
  }
}
