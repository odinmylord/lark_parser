from copy import deepcopy
from lark import Visitor, Token, Tree
from utils import children_to_string


class MessagesFinder(Visitor):
    """
    Class to find the messages exchanged between processes in a DPS file.
    """
    _processes = {}
    _current_process = None
    _actual_statement = None
    _current_channel = None
    _current_depth = 0
    _parenthesis_counter = 0
    _single_line_in_branch = False
    _func_content_counter = 0
    _parenthesis_list = []
    _channels = set()
    _channel_separator = "2"
    name_to_process_letter = {}
    variables_mapping = {}

    def set_channel_separator(self, separator: str):
        self._channel_separator = separator

    def assignment(self, tree):
        if self._current_process is None:
            self._current_process = tree.children[0].children[0].value
            self._processes[self._current_process] = []

        if tree.children[0].children[0].value == "=":
            message = tree.children[0].children[1].value
            variable_name = tree.children[-1].children[0].children[0].value
            self.variables_mapping[variable_name] = message

    def channel_declaration(self, tree):
        channels = children_to_string(tree)
        if tree.children[-1].value == "private":
            channels = channels.removesuffix("private")
        channels = [c.strip() for c in channels.split(",")]
        self._channels.update(channels)

    def statement(self, tree):
        parenthesis_found = False
        if isinstance(tree.children[0], Token):
            if tree.children[0].value == "(" and self._actual_statement in ["then",
                                                                            "else_statement"]:
                self._parenthesis_list.append(self._parenthesis_counter)
                self._parenthesis_counter += 1
                parenthesis_found = True
            elif tree.children[0].value == ")":
                self._parenthesis_counter -= 1
                if self._parenthesis_counter in self._parenthesis_list:
                    self._parenthesis_list.remove(self._parenthesis_counter)
                    self._current_depth -= 1
        if not parenthesis_found and self._actual_statement in ["then", "else_statement"] and \
                tree.children[0].data != self._actual_statement:
            self._single_line_in_branch = True
        elif parenthesis_found:
            self._actual_statement = "LPAR"
        if isinstance(tree.children[0], Tree) and tree.children[0].data != "func_content":
            self._actual_statement = tree.children[0].data

    def func_content(self, tree):
        if self._actual_statement in ["in", "out"]:
            if tree.children and tree.children[0] in self._channels:
                index = int(self._actual_statement == "in")
                self.name_to_process_letter[self._current_process] = tree.children[0].split(
                    self._channel_separator)[index]
            if self._current_channel is None and len(tree.children) == 1:
                self._current_channel = tree.children[0].value
                if self._current_channel in self._channels:
                    message_dict = {
                        "direction": self._actual_statement,
                        "send_channel": self._current_channel.split(self._channel_separator)[0],
                        "receive_channel": self._current_channel.split(self._channel_separator)[1],
                        "message": ""
                    }
                    self.add_message(message_dict)
                else:
                    self._current_channel = None
            elif self._current_channel:
                self.add_message(children_to_string(tree))
                self._current_channel = None
                if self._single_line_in_branch:
                    self._single_line_in_branch = False
                    self._current_depth -= 1

    def dot(self, _):
        self._current_process = None
        self._current_channel = None

    def if_statement(self, _):
        self._current_channel = None
        self.add_message({"if_statem": []})
        self._current_depth += 1

    def else_statement(self, _):
        self._current_channel = None
        self.add_message({"else_statem": []})
        self._current_depth += 1

    def semicolon(self, _):
        self._current_channel = None

    # def constant(self, tree):
    #     if self._current_process:
    #         self.add_message(tree.children[0].value)

    def output(self):
        return deepcopy(self._processes)

    def visit(self, tree):
        raise NotImplementedError("Use visit_topdown instead")

    def add_message(self, message):
        current_list = self._processes[self._current_process]
        if self._current_depth:
            for _ in range(self._current_depth):
                if "if_statem" in current_list[-1]:
                    current_list = current_list[-1]["if_statem"]
                else:
                    current_list = current_list[-1]["else_statem"]
        if isinstance(message, str) and message.isnumeric():
            current_list.append(message)
        elif isinstance(message, str):
            current_list[-1]["message"] += message
        elif isinstance(message, dict):
            current_list.append(message)

    def condition(self, tree):
        self._processes[self._current_process][-1]["statem"] = children_to_string(tree)


class QueryFinder(Visitor):
    _current_process = None
    _processes = {}

    def assignment(self, tree):
        if self._current_process is None:
            self._current_process = tree.children[0].children[0].value
            self._processes[self._current_process] = []

    def statement(self, tree):
        if self._current_process and isinstance(tree.children[0], Tree):
            if tree.children[0].data == "func_content":
                self._processes[self._current_process].append(
                    children_to_string(tree))
            else:
                self._processes[self._current_process].append(
                    tree.children[0].data)

    def query(self, tree):
        if self._current_process is None:
            self._current_process = "query"
            self._processes[self._current_process] = []

    def dot(self, _):
        self._current_process = None

    def output(self):
        processes = self._processes["query"][0].split(",")
        processes = [p.strip("(").strip(")") for p in processes]
        output_dict = {k: v for k, v in self._processes.items()
                       if k in processes}
        for k, v in output_dict.items():
            output_dict[k] = [x for x in v if x not in ["dot", "pipe"]]
        output_dict["query"] = processes
        return output_dict

    def visit(self, tree):
        raise NotImplementedError("Use visit_topdown instead")
