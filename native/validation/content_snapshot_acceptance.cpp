#include <fstream>
#include <iostream>
#include <map>
#include <set>
#include <sstream>
#include <string>
#include <vector>

namespace {
struct Control {
  std::string group;
  std::string action;
  bool visible = false;
  bool enabled = false;
};

struct BusinessBlock {
  std::string group;
  std::string label;
};

struct ContentField {
  std::string group;
  std::string label;
  std::string value;
  std::string state;
};

struct ContentState {
  std::string group;
  std::string value;
  bool active = false;
};

struct ContentList {
  std::string group;
  std::string items;
  std::string empty_state;
};

struct EmptyState {
  std::string group;
  std::string text;
  bool visible = false;
};

struct Snapshot {
  std::string route;
  std::map<std::string, Control> controls;
  std::map<std::string, BusinessBlock> business;
  std::map<std::string, ContentField> fields;
  std::map<std::string, ContentState> states;
  std::map<std::string, ContentList> lists;
  std::map<std::string, EmptyState> empty_states;
  std::set<std::string> groups;
  std::set<std::string> actions;
};

struct Contract {
  std::string key;
  std::string route;
  std::string filename;
  std::string business_group;
  std::string business_id;
  std::string list_id;
  std::vector<std::string> fields;
  std::string state_id;
  std::string empty_state_id;
  std::map<std::string, std::string> action_mapping;
};

std::string trim(std::string value) {
  const auto first = value.find_first_not_of(" \t\r\n");
  if (first == std::string::npos) return {};
  const auto last = value.find_last_not_of(" \t\r\n");
  return value.substr(first, last - first + 1);
}

std::vector<std::string> split(const std::string& value, char separator) {
  std::vector<std::string> result;
  std::stringstream stream(value);
  std::string item;
  while (std::getline(stream, item, separator)) result.push_back(trim(item));
  return result;
}

bool key_value(const std::string& value, std::string& key, std::string& item) {
  const auto separator = value.find('=');
  if (separator == std::string::npos) return false;
  key = trim(value.substr(0, separator));
  item = trim(value.substr(separator + 1));
  return !key.empty() && !item.empty();
}

bool parse_bool(const std::string& value) {
  return value == "1" || value == "true";
}

bool parse_snapshot(const std::string& filename, Snapshot& snapshot,
                    std::string& error) {
  std::ifstream file(filename);
  if (!file) {
    error = "cannot open snapshot: " + filename;
    return false;
  }

  std::string line;
  size_t line_number = 0;
  while (std::getline(file, line)) {
    ++line_number;
    line = trim(line);
    if (line.empty()) continue;
    if (line[0] == '#') {
      const auto parts = split(line.substr(1), '|');
      if (!parts.empty() && parts[0] == "page") {
        for (size_t i = 1; i < parts.size(); ++i) {
          std::string key;
          std::string value;
          if (!key_value(parts[i], key, value)) continue;
          if (key == "route") snapshot.route = value;
        }
      }
      continue;
    }

    const auto parts = split(line, '|');
    if (parts.empty()) continue;
    std::map<std::string, std::string> values;
    for (size_t i = 1; i < parts.size(); ++i) {
      std::string key;
      std::string value;
      if (!key_value(parts[i], key, value) || values.count(key) != 0) {
        error = "line " + std::to_string(line_number) +
                ": malformed or duplicate key";
        return false;
      }
      values.emplace(std::move(key), std::move(value));
    }

    if (parts[0] == "group") {
      const auto id = values.find("id");
      if (id == values.end() || id->second.empty()) {
        error = "line " + std::to_string(line_number) + ": group missing id";
        return false;
      }
      snapshot.groups.insert(id->second);
    } else if (parts[0] == "control") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto action = values.find("action");
      const auto visible = values.find("visible");
      const auto enabled = values.find("enabled");
      if (id == values.end() || group == values.end() || action == values.end() ||
          visible == values.end() || enabled == values.end() || id->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": control missing id/group/action/visibility";
        return false;
      }
      if (!snapshot.controls.emplace(
               id->second,
               Control{group->second, action->second, parse_bool(visible->second),
                       parse_bool(enabled->second)})
               .second) {
        error = "line " + std::to_string(line_number) + ": duplicate control " +
                id->second;
        return false;
      }
    } else if (parts[0] == "action") {
      const auto id = values.find("id");
      const auto supported = values.find("supported");
      if (id == values.end() || supported == values.end() ||
          !parse_bool(supported->second)) {
        error = "line " + std::to_string(line_number) +
                ": malformed or unsupported action";
        return false;
      }
      snapshot.actions.insert(id->second);
    } else if (parts[0] == "section" || parts[0] == "business") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto label = values.find("label");
      if (id == values.end() || group == values.end() || label == values.end() ||
          id->second.empty() || label->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": business block requires id/group/label";
        return false;
      }
      if (!snapshot.business.emplace(id->second,
                                     BusinessBlock{group->second, label->second})
               .second) {
        error = "line " + std::to_string(line_number) +
                ": duplicate business block " + id->second;
        return false;
      }
    } else if (parts[0] == "field") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto label = values.find("label");
      const auto value = values.find("value");
      const auto state = values.find("state");
      if (id == values.end() || group == values.end() || label == values.end() ||
          value == values.end() || state == values.end() || id->second.empty() ||
          label->second.empty() || value->second.empty() || state->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": field requires id/group/label/value/state";
        return false;
      }
      if (!snapshot.fields.emplace(
               id->second, ContentField{group->second, label->second, value->second,
                                        state->second})
               .second) {
        error = "line " + std::to_string(line_number) +
                ": duplicate field " + id->second;
        return false;
      }
    } else if (parts[0] == "list") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto items = values.find("items");
      const auto empty = values.find("empty");
      if (id == values.end() || group == values.end() || items == values.end() ||
          empty == values.end() || id->second.empty() || items->second.empty() ||
          empty->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": list requires id/group/items/empty";
        return false;
      }
      if (!snapshot.lists.emplace(
               id->second,
               ContentList{group->second, items->second, empty->second})
               .second) {
        error = "line " + std::to_string(line_number) +
                ": duplicate list " + id->second;
        return false;
      }
    } else if (parts[0] == "status" || parts[0] == "state") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto value = values.find("value");
      const auto active = values.find("active");
      if (id == values.end() || group == values.end() || value == values.end() ||
          active == values.end() || id->second.empty() || value->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": state requires id/group/value/active";
        return false;
      }
      if (!snapshot.states.emplace(
               id->second, ContentState{group->second, value->second,
                                        parse_bool(active->second)})
               .second) {
        error = "line " + std::to_string(line_number) +
                ": duplicate state " + id->second;
        return false;
      }
    } else if (parts[0] == "empty") {
      const auto id = values.find("id");
      const auto group = values.find("group");
      const auto text = values.find("text");
      const auto visible = values.find("visible");
      if (id == values.end() || group == values.end() || text == values.end() ||
          visible == values.end() || id->second.empty() || text->second.empty()) {
        error = "line " + std::to_string(line_number) +
                ": empty state requires id/group/text/visible";
        return false;
      }
      if (!snapshot.empty_states.emplace(
               id->second, EmptyState{group->second, text->second,
                                      parse_bool(visible->second)})
               .second) {
        error = "line " + std::to_string(line_number) +
                ": duplicate empty state " + id->second;
        return false;
      }
    }
  }
  if (snapshot.route.empty()) {
    error = "snapshot has no page route";
    return false;
  }
  return true;
}

bool parse_contract_line(const std::string& line, Contract& contract,
                         std::string& error) {
  const auto parts = split(line, '|');
  if (parts.size() != 10) {
    error = "contract rows require 10 fields";
    return false;
  }
  contract.key = parts[0];
  contract.route = parts[1];
  contract.filename = parts[2];
  contract.business_group = parts[3];
  contract.business_id = parts[4];
  contract.list_id = parts[5];
  contract.fields = split(parts[6], ',');
  contract.state_id = parts[7];
  contract.empty_state_id = parts[8];
  for (const auto& mapping : split(parts[9], ',')) {
    std::string id;
    std::string action;
    if (!key_value(mapping, id, action) || contract.action_mapping.count(id) != 0) {
      error = "contract " + contract.key + " has malformed action mapping";
      return false;
    }
    contract.action_mapping.emplace(std::move(id), std::move(action));
  }
  if (contract.key.empty() || contract.route.empty() || contract.filename.empty() ||
      contract.business_group.empty() || contract.business_id.empty() ||
      contract.list_id.empty() || contract.fields.empty() ||
      contract.state_id.empty() || contract.empty_state_id.empty() ||
      contract.action_mapping.empty()) {
    error = "contract " + contract.key +
            " must declare business group, block, fields, state, empty state, and actions";
    return false;
  }
  return true;
}

bool load_contract(const std::string& filename, std::vector<Contract>& contracts,
                   std::string& error) {
  std::ifstream file(filename);
  if (!file) {
    error = "cannot open content contract: " + filename;
    return false;
  }
  std::string line;
  size_t line_number = 0;
  std::set<std::string> keys;
  while (std::getline(file, line)) {
    ++line_number;
    line = trim(line);
    if (line.empty() || line[0] == '#') continue;
    Contract contract;
    if (!parse_contract_line(line, contract, error)) {
      error = "contract line " + std::to_string(line_number) + ": " + error;
      return false;
    }
    if (!keys.insert(contract.key).second) {
      error = "contract line " + std::to_string(line_number) +
              ": duplicate route/state " + contract.key;
      return false;
    }
    contracts.push_back(std::move(contract));
  }
  return !contracts.empty() || (error = "content contract is empty", false);
}

bool validate(const Contract& contract, const Snapshot& snapshot,
              std::string& error) {
  if (snapshot.route != contract.route) {
    error = "route mismatch: contract=" + contract.route +
            ", snapshot=" + snapshot.route;
    return false;
  }
  if (!snapshot.groups.count(contract.business_group)) {
    error = "missing business group " + contract.business_group;
    return false;
  }
  const auto business = snapshot.business.find(contract.business_id);
  if (business == snapshot.business.end() ||
      business->second.group != contract.business_group ||
      business->second.label.empty()) {
    error = "missing business block " + contract.business_id;
    return false;
  }
  const auto list = snapshot.lists.find(contract.list_id);
  if (list == snapshot.lists.end() || list->second.group != contract.business_group ||
      list->second.items.empty() || list->second.empty_state != contract.empty_state_id) {
    error = "missing business list " + contract.list_id;
    return false;
  }
  if (contract.fields.empty()) {
    error = "business contract has no fields";
    return false;
  }
  for (const auto& field : contract.fields) {
    const auto content = snapshot.fields.find(field);
    const auto control = snapshot.controls.find(field);
    if (content == snapshot.fields.end() || control == snapshot.controls.end()) {
      error = "missing business field/control " + field;
      return false;
    }
    if (content->second.group != contract.business_group ||
        content->second.label.empty() || content->second.value.empty() ||
        content->second.state.empty() ||
        control->second.group != contract.business_group ||
        !control->second.visible || !control->second.enabled) {
      error = "business field " + field +
              " must include value/state and visible enabled control in " +
              contract.business_group;
      return false;
    }
  }
  const auto state = snapshot.states.find(contract.state_id);
  if (state == snapshot.states.end() || state->second.group != "status" ||
      state->second.value.empty() || !state->second.active) {
    error = "missing active status state " + contract.state_id;
    return false;
  }
  const auto empty = snapshot.empty_states.find(contract.empty_state_id);
  if (empty == snapshot.empty_states.end() ||
      empty->second.group != contract.business_group ||
      empty->second.text.empty() || !empty->second.visible) {
    error = "missing visible empty state " + contract.empty_state_id;
    return false;
  }
  for (const auto& [control_id, action] : contract.action_mapping) {
    const auto control = snapshot.controls.find(control_id);
    if (control == snapshot.controls.end()) {
      error = "action mapping references missing control " + control_id;
      return false;
    }
    if (control->second.action != action || !snapshot.actions.count(action)) {
      error = "action mapping mismatch for " + control_id + " -> " + action;
      return false;
    }
  }
  return true;
}
}  // namespace

int main(int argc, char** argv) {
  if (argc != 5 || std::string(argv[1]) != "--contract" ||
      std::string(argv[3]) != "--root") {
    std::cerr << "usage: content_snapshot_acceptance --contract <file> --root <dir>\n";
    return 2;
  }
  std::vector<Contract> contracts;
  std::string error;
  if (!load_contract(argv[2], contracts, error)) {
    std::cerr << "content contract failed: " << error << '\n';
    return 1;
  }
  for (const auto& contract : contracts) {
    Snapshot snapshot;
    const std::string filename = std::string(argv[4]) + "/" + contract.key.substr(0, contract.key.find('/')) + "/" + contract.filename;
    if (!parse_snapshot(filename, snapshot, error) ||
        !validate(contract, snapshot, error)) {
      std::cerr << "content acceptance failed for " << contract.key << ": "
                << error << '\n';
      return 1;
    }
  }
  std::cout << "content acceptance ok: routes=" << contracts.size() << '\n';
  return 0;
}
