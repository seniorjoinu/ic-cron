## Antifragile currency token

This token represents a registry where each individual principal has a balance of unsigned integer amount of some
currency.

#### Usage

To deploy your own copy of this token add this repository as a git submodule of your project and incorporate it into
your `dfx.json`.

To integrate your canister with already deployed token canister:

* add `antifragile-currency-token-client = "0.1.3"` (or higher version) to the `dependencies` of your `Cargo.toml`
* use `antifragile_currency_token_client::api::CurrencyTokenClient` inside your integrating canister

#### Local development

From current directory type in shell `dfx deploy`