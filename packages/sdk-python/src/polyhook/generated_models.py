# DO NOT EDIT — generated from schema.json by `make schema/python`.

from __future__ import annotations

from enum import Enum
from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field, RootModel


class PolyhookSchema(RootModel[Any]):
    root: Any = Field(
        ...,
        description=(
            'Source-of-truth type definitions for the polyhook SDK. All'
            ' language-specific types (Rust, TypeScript, Go, Python, .NET) are'
            ' generated from this file.'
        ),
        title='Polyhook Schema',
    )


class CallerKind(Enum):
    claude_code = 'claude-code'
    cursor = 'cursor'
    windsurf = 'windsurf'
    cline = 'cline'
    amp = 'amp'
    gemini_cli = 'gemini-cli'
    unknown = 'unknown'


class Event(Enum):
    tool_before = 'tool:before'
    tool_after = 'tool:after'
    session_start = 'session:start'
    session_stop = 'session:stop'
    agent_stop = 'agent:stop'
    notification = 'notification'


class HookEvent(BaseModel):
    model_config = ConfigDict(
        extra='forbid',
    )
    event: Event = Field(
        ...,
        description=(
            "Normalized event kind. One of: 'tool:before' (about to run a tool),"
            " 'tool:after' (tool finished), 'session:start' (new agent session opened),"
            " 'session:stop' (agent session closed), 'agent:stop' (sub-agent returned),"
            " 'notification' (informational message, no response required)."
        ),
    )
    tool: str | None = Field(
        None,
        description=(
            "Normalized tool name (e.g. 'bash', 'write_file', 'read_file'). Present for"
            " tool:before and tool:after events; null for all other event kinds."
        ),
    )
    bin: str | None = Field(
        None,
        description=(
            "The executable being invoked. Only present for bash tool events where"
            " input.command is available. Extracted as the first non-env-assignment"
            " token of the command string (e.g. 'git' from 'GIT_DIR=.git git commit')."
            " Null for all other tool kinds."
        ),
    )
    input: dict[str, Any] | None = Field(
        None,
        description=(
            'Tool input arguments as a free-form object. Present for tool:before'
            ' events; null otherwise. The shape depends on the specific tool being'
            ' called.'
        ),
    )
    output: dict[str, Any] | None = Field(
        None,
        description=(
            'Tool output as a free-form object. Present for tool:after events; null'
            ' otherwise. The shape depends on the specific tool that produced the'
            ' output.'
        ),
    )
    sessionId: str = Field(
        ...,
        description=(
            'Opaque session identifier provided by the calling AI tool. Used to'
            ' correlate events that belong to the same agent session.'
        ),
    )
    agentId: str | None = Field(
        None,
        description=(
            'Opaque identifier for the sub-agent that triggered this event. Present'
            ' only when the hook is invoked from within a sub-agent context; null at'
            ' the top-level agent.'
        ),
    )
    caller: CallerKind = Field(
        ...,
        description=(
            "The AI coding tool that invoked this hook binary, detected from"
            " environment variables and stdin format. Defaults to 'unknown' when"
            " detection fails."
        ),
    )


class ApproveResponse(BaseModel):
    model_config = ConfigDict(
        extra='forbid',
    )
    action: Literal['approve'] = Field(
        ..., description='Discriminator field identifying this as an approve response.'
    )


class BlockResponse(BaseModel):
    model_config = ConfigDict(
        extra='forbid',
    )
    action: Literal['block'] = Field(
        ..., description='Discriminator field identifying this as a block response.'
    )
    message: str = Field(
        ...,
        description=(
            'Human-readable explanation shown to the user explaining why the operation'
            ' was blocked. Should be clear and actionable.'
        ),
    )


class ModifyResponse(BaseModel):
    model_config = ConfigDict(
        extra='forbid',
    )
    action: Literal['modify'] = Field(
        ..., description='Discriminator field identifying this as a modify response.'
    )
    input: dict[str, Any] = Field(
        ...,
        description=(
            'Replacement input arguments to use instead of the original tool input. The'
            ' shape must be compatible with the tool being called.'
        ),
    )


class HookResponse(RootModel[ApproveResponse | BlockResponse | ModifyResponse]):
    root: ApproveResponse | BlockResponse | ModifyResponse = Field(
        ...,
        description=(
            "The response a hook handler returns to polyhook.wasm, which translates it"
            " into the format expected by the detected caller. Discriminated on the"
            " 'action' field."
        ),
        title='HookResponse',
    )
