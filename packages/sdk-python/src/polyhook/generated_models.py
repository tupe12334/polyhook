# DO NOT EDIT — generated from schema.json by `make schema/python`.

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any, Literal, TypeAlias

PolyhookSchema: TypeAlias = Any


class CallerKind(Enum):
    claude_code = 'claude-code'
    cursor = 'cursor'
    windsurf = 'windsurf'
    cline = 'cline'
    amp = 'amp'
    gemini_cli = 'gemini-cli'
    hermes = 'hermes'
    unknown = 'unknown'


class Event(Enum):
    tool_before = 'tool:before'
    tool_after = 'tool:after'
    session_start = 'session:start'
    session_stop = 'session:stop'
    agent_stop = 'agent:stop'
    notification = 'notification'


@dataclass
class HookEvent:
    event: Event
    sessionId: str
    caller: CallerKind
    tool: str | None = None
    input: dict[str, Any] | None = None
    output: dict[str, Any] | None = None
    agentId: str | None = None


@dataclass
class ApproveResponse:
    action: Literal['approve']


@dataclass
class BlockResponse:
    action: Literal['block']
    message: str


@dataclass
class ModifyResponse:
    action: Literal['modify']
    input: dict[str, Any]


HookResponse: TypeAlias = ApproveResponse | BlockResponse | ModifyResponse
