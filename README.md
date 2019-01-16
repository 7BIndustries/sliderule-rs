# sliderule-rs

[![Travis Build Status](https://travis-ci.org/7BIndustries/sliderule-rs.svg?branch=master)](https://travis-ci.org/7BIndustries/sliderule-rs)
[![Appveyor Build Status](https://ci.appveyor.com/api/projects/status/b2cvxvvv8irflqgu/branch/master?svg=true)](https://ci.appveyor.com/project/jmwright/sliderule-rs)
[![codecov](https://codecov.io/gh/7BIndustries/sliderule-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/7BIndustries/sliderule-rs)

## Introduction
Rust crate that encapsulates an implementation of the Distributed OSHW (Open Source Hardware) Framework [DOF](https://github.com/Mach30/dof) being developed by [Mach 30](http://mach30.org/).

## Running Tests

If [Rust is installed](https://www.rust-lang.org/en-US/install.html), running the following command will execute the tests.
```
cargo test
```

## Developers

This is a section for developers who want to use this crate.

### Exit Codes
#### General
- 1 - The component being accessed is not associated with a repository
#### Git
- 100 - Could not open local repository directory
- 101 - Could not find remote reference
- 102 - Could not download from remote
- 103 - Could not update the references to point to the right commits
#### npm
