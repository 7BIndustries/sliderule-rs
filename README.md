# sliderule-rs

[![Travis Build Status](https://travis-ci.org/7BIndustries/sliderule-rs.svg?branch=master)](https://travis-ci.org/7BIndustries/sliderule-rs)
[![Appveyor Build Status](https://ci.appveyor.com/api/projects/status/b2cvxvvv8irflqgu/branch/master?svg=true)](https://ci.appveyor.com/project/jmwright/sliderule-rs)
[![codecov](https://codecov.io/gh/7BIndustries/sliderule-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/7BIndustries/sliderule-rs)
[![](http://meritbadge.herokuapp.com/sliderule)](https://crates.io/crates/sliderule)

## Introduction
This Rust crate encapsulates an implementation of the Distributed OSHW (Open Source Hardware) Framework [DOF](https://github.com/Mach30/dof) being developed by [Mach 30](http://mach30.org/).

Sliderule wraps the `git` and `npm` commands and uses them to manage DOF/Sliderule projects, both on the local file system, and on a remote server. At this time only structure management is provided, there is no capability to render models for documentation like assembly instructions out into their distributable form.

Central to understanding Sliderule is the concept of _local_ and _remote_ components. _Local_ components are stored with a project, which is the top-level, enclosing component. Local components do not have a repository associated with them, they are only stored in the `components` directory of a project. Remote components, on the other hand, are stored in a remote repository, and are only installed into the local file system if the user requests it. Remote components are intended to be shared, local components are intended to only be used within their parent project. A local component can be converted into a remote component later, if desired.

The following is a list of the major operations available through this crate.
- _create_ - Creates a top level component unless creating a component inside of an existing component. In that case the new component is placed within the `components` directory of the parent "project" component.
- _add_ - Adds a remote component from a repository into the `node_modules` directory of the current component.
- _download_ - Downloads a copy of a component form a remote repository.
- _update_ - Downloads the latest changes to the component and/or its remote components (dependencies)
- _remove_ - Removes a component, whether it is local or remote.
- _upload_ - Uploads component changes on the local file system to its remote repository.
- _refactor_ - Converts a local component to a remote component so that it may be more easily shared.

There are also various helper functions to do things like getting what level a component is in a hierarchy and compiling the licenses of all components in a project.

## API

This readme is just a general overview. The [API documentation](https://docs.rs/sliderule/0.1.0/sliderule/) for this crate is available on crates.io.

## Running Tests

If [Rust is installed](https://www.rust-lang.org/en-US/install.html), running the following command will execute the tests.
```
cargo test -- --test-threads=1
```
