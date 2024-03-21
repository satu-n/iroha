import allure  # type: ignore
import pytest

from src.client_cli import client_cli, have, iroha


@pytest.fixture(scope="function", autouse=True)
def story_account_registers_trigger():
    allure.dynamic.story("Account register a register_trigger")
    allure.dynamic.label("permission", "no_permission_required")


@allure.label("sdk_test_id", "register_trigger")
@pytest.mark.xfail(reason="wait for #4151")
def test_register_trigger(GIVEN_currently_authorized_account):
    with allure.step(
        f'WHEN client_cli registers a register_trigger for "{GIVEN_currently_authorized_account}"'
    ):
        client_cli.register_trigger(GIVEN_currently_authorized_account)
    with allure.step(
        "THEN Iroha should have the asset with nft_number_1_for_genesis##\
        ed0120E2ECD69DA5833EC10FB3DFAED83A07E5B9CBE9BC39484F0F7DDEC8B46253428B@genesis"
    ):
        iroha.should(
            have.asset(
                "nft_number_1_for_genesis##\
                ed0120E2ECD69DA5833EC10FB3DFAED83A07E5B9CBE9BC39484F0F7DDEC8B46253428B@genesis"
            )
        )
