#pragma once

#include <vector>
#include <mutex>

#include "int_types.h"
#include "via.h"


extern std::mutex solutionMutex;


typedef std::vector<ViaLayer> RouteStepVec;
typedef std::vector<RouteStepVec> RouteVec;
typedef std::vector<std::string> SolutionInfoVec;

class Solution
{
public:
  Solution();
  RouteVec& getRouteVec();
  SolutionInfoVec& getSolutionInfoVec();
  void dump();
  void setErrorBool(bool errorBool);
  bool getErrorBool();
private:
  SolutionInfoVec solutionInfoVec_;
  RouteVec routeVec_;
  bool hasError_;
};