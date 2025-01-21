# ckb_ssri_sdk

## This crate is not officially released yet!

> [[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256): General introduction to SSRI.
>
> [CCC App - SSRI](https://ccc-git-udtssridemo-aaaaaaaalive24.vercel.app/?_vercel_share=zQkvWcsB2U9HRbpRFtF9w3xQT9msZDWb): Interactive demo of interacting with SSRI-Compliant scripts.
>
> [`pausable-udt`](https://github.com/ckb-devrel/pausable-udt): A real production level contract (instead of a pseudo-project) that exemplifies the usage of SSRI.
>
> [`ssri-server`](https://github.com/ckb-devrel/ssri-server): Server for calling SSRI methods.

## Quick Note on SSRI

- SSRI stands for `Script Sourced Rich Information`; it is a protocol for strong bindings of relevant information and conventions to the Script itself on CKB. For more information, please read [[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256)>.
- For writing CKB Scripts (or "Smart Contracts"), by selectively implementing methods of public module traits (e.g. `UDT`, `UDTExtended`, `UDTPausable`) in combinations, devs would be able to quickly design and organize functionalities that either validate transactions or provide rich information as well as assembling transactions off-chain.
- For dApps or other infrastructures that interact with CKB Scripts, you no longer need to retrieve and parse data or assemble transactions by yourself repetitively as they are all provided by SSRI.

### Goals of `ckb_ssri_sdk`

- Easier and intuitive implementations and built-in integration support of SSRI public traits which would receive first party support such as [`@ccc-ckb/udt`](https://docs.ckbccc.com/modules/_ckb_ccc_udt.html).
- Easier and intuitive definitions of customized SSRI traits.
- [ ] TODO: Unified entry function;

## Usage: Example Contract

[`pausable-udt`](https://github.com/ckb-devrel/pausable-udt) is a real production level contract (instead of a pseudo-project) that exemplifies the usage of SSRI.

## Testing

Due to the limitations of `ckb_testtools`, it is recommended to test the same SSRI-Compliant Contract on two level:

- On-chain Verification: Test with `ckb_testtools`
- Off-chain Query/Integration, Transaction Generations/Completions: Test with `ckb_ssri_cli` against the latest deployment.
