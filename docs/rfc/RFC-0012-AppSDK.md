# RFC-0012-AppSDK.md

Status: Accepted
Version: 1.0

## Abstract

AppSDK is the official developer platform. It includes RetroKit, RetroBus, LaunchServices API, Settings API, Storage API, and Media APIs.

## Structure

MyApp.app/
    App.toml
    Executable/
    Resources/

## Developer Workflow

Create app → build bundle → run bundle → ship bundle.

## SDK Goals

Stable APIs, documentation, consistency, portability.

## Non-Goals

No web runtime, browser UI, or Electron compatibility.

## Required Behavior

Applications must support themes, accessibility, keyboard navigation, global menu bar, and WindowManager.

## Future Tools

Interface Builder, Bundle Generator, Localization Tools, Theme Validator, Application Profiler.
