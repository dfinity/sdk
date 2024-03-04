from kybra import query

@query
def greet(name: str) -> str:
    return f"Hello, {name}!"
