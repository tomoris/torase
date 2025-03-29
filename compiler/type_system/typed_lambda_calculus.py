from dataclasses import dataclass


class Type:
    pass


@dataclass
class TInt(Type):
    def __str__(self):
        return "Int"


@dataclass
class TBool(Type):
    def __str__(self):
        return "Bool"


@dataclass
class Arrow(Type):
    left: Type
    right: Type

    def __str__(self):
        return f"({self.left} → {self.right})"


@dataclass
class Argument:
    pass


@dataclass
class Variable(Argument):
    name: str

    def __str__(self):
        return self.name


@dataclass
class LambdaAbstraction(Argument):
    param: str
    param_type: Type
    body: Argument

    def __str__(self):
        return f"(λ{self.param}:{self.param_type}.{self.body})"


@dataclass
class FunctionApplication(Argument):
    func: Argument
    arg: Argument

    def __str__(self):
        return f"({self.func} {self.arg})"


@dataclass
class Constant(Argument):
    value: int | bool

    def __str__(self):
        return str(self.value)


@dataclass
class If(Argument):
    cond: Argument
    then_branch: Argument
    else_branch: Argument

    def __str__(self):
        return f"(if {self.cond} then {self.then_branch} else {self.else_branch})"


TypeEnvironment = dict[str, Type]
Context = dict[str, Argument]


class TypedLambdaTypeError(Exception):
    def __init__(self, message: str):
        super().__init__(message)
        self.message = message

    def __str__(self) -> str:
        return self.message


class TypedLambdaCalculator:
    def __init__(self, type_env: TypeEnvironment | None = None):
        if type_env is None:
            self.type_env: TypeEnvironment = {}
        else:
            self.type_env = type_env
        self.ctx: Context = {}

    def check_type(self, arg: Argument) -> Type:
        match arg:
            case Constant():
                return TBool() if isinstance(arg.value, bool) else TInt()
            case Variable():
                if arg.name not in self.type_env:
                    raise TypedLambdaTypeError(f"Undefined variable: {arg.name}")
                return self.type_env[arg.name]
            case LambdaAbstraction():
                self.type_env[arg.param] = arg.param_type
                body_type = self.check_type(arg.body)
                del self.type_env[arg.param]
                return Arrow(arg.param_type, body_type)
            case FunctionApplication():
                func_type = self.check_type(arg.func)
                arg_type = self.check_type(arg.arg)
                if isinstance(func_type, Arrow):
                    if func_type.left == arg_type:
                        return func_type.right
                    else:
                        raise TypedLambdaTypeError(
                            f"Type mismatch: {func_type} and {arg_type}"
                        )
                else:
                    raise TypedLambdaTypeError(
                        f"Cannot apply non-function: {func_type}"
                    )
            case If():
                cond_type = self.check_type(arg.cond)
                if not isinstance(cond_type, TBool):
                    raise TypedLambdaTypeError(
                        f"Condition must be a boolean: {cond_type}"
                    )
                then_type = self.check_type(arg.then_branch)
                else_type = self.check_type(arg.else_branch)
                if then_type != else_type:
                    raise TypedLambdaTypeError(
                        f"Branches must have the same type: {then_type} and {else_type}"
                    )
                return then_type
            case _:
                raise TypedLambdaTypeError(f"Unknown argument type: {type(arg)}")

    def evaluate(self, arg: Argument) -> Argument:
        match arg:
            case Constant():
                return arg
            case Variable():
                if arg.name not in self.ctx:
                    raise TypedLambdaTypeError(f"Undefined variable: {arg.name}")
                return self.evaluate(self.ctx[arg.name])
            case LambdaAbstraction():
                return arg
            case FunctionApplication():
                func = self.evaluate(arg.func)
                if isinstance(func, LambdaAbstraction):
                    new_arg = self.evaluate(arg.arg)
                    self.ctx[func.param] = new_arg
                    result = self.evaluate(func.body)
                    del self.ctx[func.param]
                    return result
                else:
                    raise TypedLambdaTypeError(f"Cannot apply non-function: {func}")
            case If():
                cond = self.evaluate(arg.cond)
                if not isinstance(cond, Constant) or not isinstance(cond.value, bool):
                    raise TypedLambdaTypeError(f"Condition must be a boolean: {cond}")
                if cond.value:
                    return self.evaluate(arg.then_branch)
                else:
                    return self.evaluate(arg.else_branch)
            case _:
                raise TypedLambdaTypeError(f"Unknown argument type: {type(arg)}")
