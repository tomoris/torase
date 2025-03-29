from compiler.parsing.my_ast import (
    AbstractSyntaxTree,
    EPSILON_SYMBOL,
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
    ):
        self.app_priorities = app_priorities
        self.arg_definitions = arg_definitions

    def _get_arg_def(self, s: Symbol | Epsilon) -> Argument:
        if s not in self.arg_definitions:
            raise UnknownSymbolError(f"Unknown symbol's argument: {s}")
        return self.arg_definitions[s]

    def convert(self, ast: AbstractSyntaxTree) -> Argument:
        if ast.children == EPSILON_SYMBOL:
            return FunctionApplication(
                self._get_arg_def(ast.non_terminal_symbol),
                self._get_arg_def(EPSILON_SYMBOL),  # type: ignore[arg-type]
            )

        # 子がlistの場合
        child_args = []
        for child in ast.children:
            if isinstance(child, AbstractSyntaxTree):
                child_args.append(self.convert(child))
            else:  # TerminalSymbol
                child_args.append(self._get_arg_def(child))  # type: ignore[arg-type]
        if len(child_args) == 1:
            return FunctionApplication(
                self._get_arg_def(ast.non_terminal_symbol),
                child_args[0],
            )

        # 子がlistで2つ以上の場合
        production_tuple = tuple(
            [
                c.non_terminal_symbol if isinstance(c, AbstractSyntaxTree) else c
                for c in ast.children
            ]
        )
        prior = self.app_priorities[production_tuple]

        return FunctionApplication(
            self._get_arg_def(ast.non_terminal_symbol),
            self._app(prior, child_args),
        )

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
