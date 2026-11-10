#include <algorithm>
#include <cctype>
#include <fstream>
#include <iostream>
#include <map>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {
struct Rect {
  int x = 0;
  int y = 0;
  int w = 0;
  int h = 0;
};

struct Group {
  Rect rect;
  int left = -1;
  int top = -1;
  int right = -1;
  int bottom = -1;
};

struct Control {
  std::string group;
  Rect rect;
  bool visible = false;
  bool enabled = false;
};

struct Snapshot {
  Rect window;
  bool has_window = false;
  std::map<std::string, Group> groups;
  std::map<std::string, Control> controls;
  std::set<std::string> actions;
};

std::string trim(std::string value) {
  const auto first = value.find_first_not_of(" \t\r\n");
  if (first == std::string::npos) return {};
  const auto last = value.find_last_not_of(" \t\r\n");
  return value.substr(first, last - first + 1);
}

bool parse_int(const std::map<std::string, std::string>& values,
               const std::string& key, int& result, std::string& error) {
  const auto found = values.find(key);
  if (found == values.end()) {
    error = "missing " + key;
    return false;
  }
  try {
    size_t consumed = 0;
    result = std::stoi(found->second, &consumed);
    if (consumed != found->second.size()) throw std::invalid_argument("trailing");
  } catch (...) {
    error = "invalid integer for " + key;
    return false;
  }
  return true;
}

bool parse_bool(const std::map<std::string, std::string>& values,
                const std::string& key, bool& result, std::string& error) {
  const auto found = values.find(key);
  if (found == values.end()) {
    error = "missing " + key;
    return false;
  }
  if (found->second == "1" || found->second == "true") result = true;
  else if (found->second == "0" || found->second == "false") result = false;
  else {
    error = "invalid boolean for " + key;
    return false;
  }
  return true;
}

bool parse_rect(const std::map<std::string, std::string>& values, Rect& rect,
                std::string& error) {
  return parse_int(values, "x", rect.x, error) &&
         parse_int(values, "y", rect.y, error) &&
         parse_int(values, "w", rect.w, error) &&
         parse_int(values, "h", rect.h, error);
}

bool parse_line(const std::string& raw, size_t line_number, Snapshot& snapshot,
                std::string& error) {
  const auto line = trim(raw);
  if (line.empty() || line[0] == '#') return true;

  std::vector<std::string> parts;
  std::stringstream stream(line);
  std::string part;
  while (std::getline(stream, part, '|')) parts.push_back(trim(part));
  if (parts.empty()) return true;

  std::map<std::string, std::string> values;
  for (size_t i = 1; i < parts.size(); ++i) {
    const auto separator = parts[i].find('=');
    if (separator == std::string::npos) {
      error = "line " + std::to_string(line_number) + ": malformed key/value";
      return false;
    }
    const auto key = trim(parts[i].substr(0, separator));
    const auto value = trim(parts[i].substr(separator + 1));
    if (key.empty() || values.find(key) != values.end()) {
      error = "line " + std::to_string(line_number) + ": duplicate/empty key";
      return false;
    }
    values[key] = value;
  }

  if (parts[0] == "window") {
    if (snapshot.has_window) {
      error = "line " + std::to_string(line_number) + ": duplicate window";
      return false;
    }
    if (!parse_rect(values, snapshot.window, error)) return false;
    snapshot.has_window = true;
    return true;
  }
  if (parts[0] == "group") {
    const auto id = values.find("id");
    if (id == values.end() || id->second.empty()) {
      error = "line " + std::to_string(line_number) + ": missing group id";
      return false;
    }
    Group group;
    if (!parse_rect(values, group.rect, error)) return false;
    if (!parse_int(values, "left", group.left, error) ||
        !parse_int(values, "top", group.top, error) ||
        !parse_int(values, "right", group.right, error) ||
        !parse_int(values, "bottom", group.bottom, error)) return false;
    if (!snapshot.groups.emplace(id->second, group).second) {
      error = "line " + std::to_string(line_number) + ": duplicate group id";
      return false;
    }
    return true;
  }
  if (parts[0] == "control") {
    const auto id = values.find("id");
    const auto group = values.find("group");
    if (id == values.end() || id->second.empty() || group == values.end() || group->second.empty()) {
      error = "line " + std::to_string(line_number) + ": missing control id/group";
      return false;
    }
    Control control;
    control.group = group->second;
    if (!parse_rect(values, control.rect, error) ||
        !parse_bool(values, "visible", control.visible, error) ||
        !parse_bool(values, "enabled", control.enabled, error)) return false;
    if (!snapshot.controls.emplace(id->second, control).second) {
      error = "line " + std::to_string(line_number) + ": duplicate control id";
      return false;
    }
    return true;
  }
  if (parts[0] == "action") {
    const auto id = values.find("id");
    const auto supported = values.find("supported");
    if (id == values.end() || id->second.empty() || supported == values.end() ||
        (supported->second != "1" && supported->second != "true")) {
      error = "line " + std::to_string(line_number) + ": unsupported/malformed action";
      return false;
    }
    snapshot.actions.insert(id->second);
    return true;
  }
  if (parts[0] == "section" || parts[0] == "field" || parts[0] == "list" ||
      parts[0] == "status" || parts[0] == "empty") {
    if (values.find("id") == values.end() || values.at("id").empty()) {
      error = "line " + std::to_string(line_number) + ": content record missing id";
      return false;
    }
    return true;
  }
  error = "line " + std::to_string(line_number) + ": unknown record " + parts[0];
  return false;
}

bool contains(const Rect& outer, const Rect& inner) {
  return inner.x >= outer.x && inner.y >= outer.y &&
         inner.x + inner.w <= outer.x + outer.w &&
         inner.y + inner.h <= outer.y + outer.h;
}

bool overlaps(const Rect& a, const Rect& b) {
  return a.x < b.x + b.w && a.x + a.w > b.x &&
         a.y < b.y + b.h && a.y + a.h > b.y;
}

bool validate(const Snapshot& snapshot, std::string& error) {
  if (!snapshot.has_window || snapshot.window.w <= 0 || snapshot.window.h <= 0) {
    error = "window must have positive dimensions";
    return false;
  }
  if (snapshot.groups.empty()) {
    error = "at least one group is required";
    return false;
  }
  if (snapshot.controls.empty()) {
    error = "at least one control is required";
    return false;
  }
  if (snapshot.actions.empty()) {
    error = "at least one supported action is required";
    return false;
  }

  for (const auto& [id, group] : snapshot.groups) {
    if (group.rect.w <= 0 || group.rect.h <= 0 || !contains(snapshot.window, group.rect)) {
      error = "group " + id + " is outside the window or has invalid size";
      return false;
    }
    if (group.left < 0 || group.top < 0 || group.right < 0 || group.bottom < 0 ||
        group.left + group.right >= group.rect.w || group.top + group.bottom >= group.rect.h) {
      error = "group " + id + " has invalid margins";
      return false;
    }
  }

  std::map<std::string, std::vector<Rect>> controls_by_group;
  for (const auto& [id, control] : snapshot.controls) {
    const auto group = snapshot.groups.find(control.group);
    if (group == snapshot.groups.end()) {
      error = "control " + id + " references unknown group " + control.group;
      return false;
    }
    if (control.rect.w <= 0 || control.rect.h <= 0 || !control.visible || !control.enabled) {
      error = "control " + id + " must be visible, enabled, and non-empty";
      return false;
    }
    const Rect content{
        group->second.rect.x + group->second.left,
        group->second.rect.y + group->second.top,
        group->second.rect.w - group->second.left - group->second.right,
        group->second.rect.h - group->second.top - group->second.bottom,
    };
    if (!contains(content, control.rect)) {
      error = "control " + id + " violates group margins";
      return false;
    }
    controls_by_group[control.group].push_back(control.rect);
  }

  for (const auto& [group, controls] : controls_by_group) {
    for (size_t i = 0; i < controls.size(); ++i) {
      for (size_t j = i + 1; j < controls.size(); ++j) {
        if (overlaps(controls[i], controls[j])) {
          error = "controls overlap in group " + group;
          return false;
        }
      }
    }
  }
  return true;
}
}  // namespace

int main(int argc, char** argv) {
  std::ifstream file;
  std::istream* input = &std::cin;
  if (argc > 2) {
    std::cerr << "usage: layout_snapshot_smoke [snapshot-file]\n";
    return 2;
  }
  if (argc == 2) {
    file.open(argv[1]);
    if (!file) {
      std::cerr << "cannot open snapshot: " << argv[1] << '\n';
      return 2;
    }
    input = &file;
  }

  Snapshot snapshot;
  std::string line;
  size_t line_number = 0;
  std::string error;
  while (std::getline(*input, line)) {
    ++line_number;
    if (!parse_line(line, line_number, snapshot, error)) {
      std::cerr << "snapshot parse failed: " << error << '\n';
      return 1;
    }
  }
  if (!validate(snapshot, error)) {
    std::cerr << "snapshot validation failed: " << error << '\n';
    return 1;
  }
  std::cout << "layout snapshot ok: groups=" << snapshot.groups.size()
            << " controls=" << snapshot.controls.size()
            << " actions=" << snapshot.actions.size() << '\n';
  return 0;
}
