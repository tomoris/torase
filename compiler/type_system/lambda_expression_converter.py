from compiler.parsing.grammar import Grammar
from compiler.parsing.my_ast import (
    EPSILON_SYMBOL,
    AbstractSyntaxTree,
    Epsilon,
    Symbol,
)
from compiler.type_system.typed_lambda_calculus import (
    Argument,
    FunctionApplication,
)

ApplicationPriorities = dict[tuple, tuple]


class UnknownSymbolError(Exception):
    def __init__(self, message: str):
        super().__init__(message)
        self.message = message

    def __str__(self) -> str:
        return self.message


class LambdaExpressionConverter:
    def __init__(
        self,
        app_priorities: ApplicationPriorities,
        arg_definitions: dict[Symbol | Epsilon, Argument],
        grammar: Grammar,
    ):
        self.app_priorities = app_priorities
        self.arg_definitions = arg_definitions
        self.grammar = grammar

        self._validate()

    def _validate(self) -> None:
        for lhs, rhs in self.grammar.rules.items():
            if EPSILON_SYMBOL in rhs and lhs not in self.arg_definitions:
                raise UnknownSymbolError(
                    f"Invalid arg_definitions: {lhs} is not defined in the arg_definitions"
                )
            if isinstance(rhs, list):
                for production in rhs:
                    if isinstance(production, list):
                        if len(production) >= 2:
                            production_tuple = tuple(production)
                            if production_tuple not in self.app_priorities:
                                raise UnknownSymbolError(
                                    f"Invalid application priority: {production_tuple} is not defined"
                                )
            for terminal_symbol in self.grammar.terminals:
                if terminal_symbol not in self.arg_definitions:
                    raise UnknownSymbolError(
                        f"Invalid arg_definitions: {terminal_symbol} is not defined"
                    )

    def _get_arg_def(self, s: Symbol | Epsilon) -> Argument:
        if s not in self.arg_definitions:
            raise UnknownSymbolError(f"Unknown symbol's argument: {s}")
        return self.arg_definitions[s]

    def convert(self, ast: AbstractSyntaxTree) -> Argument:
        if ast.children == EPSILON_SYMBOL:
            return self._get_arg_def(ast.non_terminal_symbol)

        # 子がlistの場合
        child_args = []
        for child in ast.children:
            if isinstance(child, AbstractSyntaxTree):
                child_args.append(self.convert(child))
            else:  # TerminalSymbol
                child_args.append(self._get_arg_def(child))  # type: ignore[arg-type]
        if len(child_args) == 1:
            return child_args[0]

        # 子がlistで2つ以上の場合
        production_tuple = tuple(
            [
                c.non_terminal_symbol if isinstance(c, AbstractSyntaxTree) else c
                for c in ast.children
            ]
        )
        prior = self.app_priorities[production_tuple]

        return self._app(prior, child_args)

    def _app(self, prior: tuple, child_args: list[Argument]) -> FunctionApplication:
        assert len(prior) == 2
        return FunctionApplication(
            (
                child_args[prior[0]]
                if isinstance(prior[0], int)
                else self._app(prior[0], child_args)
            ),
            (
                child_args[prior[1]]
                if isinstance(prior[1], int)
                else self._app(prior[1], child_args)
            ),
        )
