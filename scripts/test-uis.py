"""
Automate frontend tests by using Playwright.
The script tests the following UIs:

1. Frontend UI.
2. Candid UI.

Examples:

$ python3 test-uis.py --frontend_url '...' --browser chromium firefox webkit  # Only test the frontend UI
$ python3 test-uis.py --candid_url '...' --browser chromium firefox webkit  # Only test the Candid UI
$ python3 test-uis.py --frontend_url '...' --candid_url '...' --browser chromium firefox webkit  # Test both UIs
"""
import argparse
import logging
import re
import sys
import time
from enum import Enum

from playwright.sync_api import sync_playwright

_CHROMIUM_BROWSER = "chromium"
_FIREFOX_BROWSER = "firefox"
_WEBKIT_BROWSER = "webkit"
_SUPPORTED_BROWSERS = {
    _CHROMIUM_BROWSER,
    _FIREFOX_BROWSER,
    _WEBKIT_BROWSER,
}
_CANDID_UI_WARNINGS_TO_IGNORE = [
    ("Error", "/index.js"),
    ("Invalid asm.js: Unexpected token", "/index.js"),
    ("Expected to find result for path [object Object], but instead found nothing.", "/index.js"),
    (
        """
Error: Server returned an error:
  Code: 404 (Not Found)
  Body: Custom section name not found.

    at j.readState (http://localhost:4943/index.js:2:11709)
    at async http://localhost:4943/index.js:2:97683
    at async Promise.all (index 0)
    at async Module.UA (http://localhost:4943/index.js:2:98732)
    at async Object.getNames (http://localhost:4943/index.js:2:266156)
    at async http://localhost:4943/index.js:2:275479""".strip(),
        "/index.js",
    ),
    (
        """
Error: Server returned an error:
  Code: 404 (Not Found)
  Body: Custom section name not found.""".strip(),
        "/index.js",
    ),
]
_CANDID_UI_ERRORS_TO_IGNORE = [
    ("Failed to load resource: the server responded with a status of 404 (Not Found)", "/read_state"),
]
# `page.route` does not support additional function parameters
_FRONTEND_URL = None


class _UI(Enum):
    CANDID = 1
    FRONTEND = 2


def _validate_browsers(browser):
    if browser not in _SUPPORTED_BROWSERS:
        logging.error(f"Browser {browser} not supported")
        sys.exit(1)

    return browser


def _get_argument_parser():
    parser = argparse.ArgumentParser(description="Test the Frontend and Candid UIs")

    parser.add_argument("--frontend_url", help="Frontend UI url")
    parser.add_argument("--candid_url", help="Candid UI url")

    parser.add_argument(
        "--browsers",
        nargs="+",
        type=_validate_browsers,
        help=f"Test against the specified browsers ({_SUPPORTED_BROWSERS})",
    )

    return parser


def _validate_args(args):
    has_err = False

    if not args.frontend_url and not args.candid_url:
        logging.error('Either "--frontend_url" or "--candid_url" must be specified to start the tests')
        has_err = True

    if not args.browsers:
        logging.error("At least one browser must be specified")
        logging.error(f"Possible browsers: {_SUPPORTED_BROWSERS}")
        has_err = True

    if has_err:
        sys.exit(1)


def _get_browser_obj(playwright, browser_name):
    if browser_name == _CHROMIUM_BROWSER:
        return playwright.chromium
    if browser_name == _FIREFOX_BROWSER:
        return playwright.firefox
    if browser_name == _WEBKIT_BROWSER:
        return playwright.webkit

    return None


def _check_console_logs(console_logs):
    logging.info("Checking console logs")

    has_err = False
    for log in console_logs:
        if log.type not in {"warning", "error"}:
            continue

        # Skip all `Error with Permissions-Policy header: Unrecognized feature` warnings
        perm_policy_warn = "Error with Permissions-Policy header:"
        if perm_policy_warn in log.text:
            logging.warning(f'Skipping Permissions-Policy warning. log.text="{log.text}"')
            continue

        url = log.location.get("url")
        if not url:
            raise RuntimeError(
                f'Cannot find "url" during log parsing (log.type={log.type}, log.text="{log.text}", log.location="{log.location}")'
            )

        for actual_text, endpoint in (
            _CANDID_UI_ERRORS_TO_IGNORE if log.type == "error" else _CANDID_UI_WARNINGS_TO_IGNORE
        ):
            if actual_text == log.text.strip() and endpoint in url:
                logging.warning(
                    f'Found {log.type}, but it was expected (log.type="{actual_text}", endpoint="{endpoint}")'
                )
                break
        else:
            logging.error(f'Found unexpected console log {log.type}. Text: "{log.text}", url: {url}')

            has_err = True

    if has_err:
        raise RuntimeError("Console has unexpected warnings and/or errors. Check previous logs")

    logging.info("Console logs are ok")


def _click_button(page, button):
    logging.info(f'Clicking button "{button}"')
    page.get_by_role("button", name=button).click()


def _set_text(page, text, value):
    logging.info(f'Setting text to "{value}"')
    page.get_by_role("textbox", name=text).fill(value)


def _test_frontend_ui_handler(page):
    # Set the name & Click the button
    name = "my name"
    logging.info(f'Setting name "{name}"')
    page.get_by_label("Enter your name:").fill(name)
    _click_button(page, "Click Me!")

    # Check if `#greeting` is populated correctly
    greeting_id = "#greeting"
    timeout_ms = 60000
    greeting_obj = page.wait_for_selector(greeting_id, timeout=timeout_ms)
    if greeting_obj:
        actual_value = greeting_obj.inner_text()
        expected_value = f"Hello, {name}!"
        if actual_value == expected_value:
            logging.info(f'"{actual_value}" found in "{greeting_id}"')
        else:
            raise RuntimeError(f'Expected greeting message is "{expected_value}", but found "{actual_value}"')
    else:
        raise RuntimeError(f"Cannot find {greeting_id} selector")


def _test_candid_ui_handler(page):
    # Set the text & Click the "Query" button
    text = "hello, world"
    _set_text(page, "text", text)
    _click_button(page, "Query")

    # Check if `#output-list` is populated correctly (after the first click)
    output_list_id = "#output-list"
    timeout_ms = 60000
    _ = page.wait_for_selector(output_list_id, timeout=timeout_ms)

    # Reset the text & Click the "Random" button
    _set_text(page, "text", "")
    _click_button(page, "Random")
    # ~

    # Check if `#output-list` is populated correctly  (after the second click)
    #
    # NOTE: `#output-list` already exists, so `wait_for_selector` won't work as expected.
    #       We noticed that, especially for `Ubuntu 20.04` and `Webkit`, the two additional lines
    #       created once the `Random` button was clicked, were not created properly.
    #
    #       For this reason there is this simple fallback logic that tries to look at the selector
    #       for more than once by sleeping for some time.
    fallback_retries = 10
    fallback_sleep_sec = 5
    last_err = None
    for _ in range(fallback_retries):
        try:
            output_list_obj = page.wait_for_selector(output_list_id, timeout=timeout_ms)
            if not output_list_obj:
                raise RuntimeError(f"Cannot find {output_list_id} selector")

            output_list_lines = output_list_obj.inner_text().split("\n")
            actual_num_lines, expected_num_lines = len(output_list_lines), 4
            if actual_num_lines != expected_num_lines:
                err = [f"Expected {expected_num_lines} lines of text but found {actual_num_lines}"]
                err.append("Lines:")
                err.extend(output_list_lines)
                raise RuntimeError("\n".join(err))

            # Extract random text from third line
            random_text = re.search(r'"([^"]*)"', output_list_lines[2])
            if not random_text:
                raise RuntimeError(f"Cannot extract the random text from the third line: {output_list_lines[2]}")
            random_text = random_text.group(1)

            for i, text_str in enumerate([text, random_text]):
                line1, line2 = (i * 2), (i * 2 + 1)

                # First output line
                actual_line, expected_line = output_list_lines[line1], f'› greet("{text_str}")'
                if actual_line != expected_line:
                    raise RuntimeError(f"Expected {expected_line} line, but found {actual_line} (line {line1})")
                logging.info(f'"{actual_line}" found in {output_list_id} at position {line1}')

                # Second output line
                actual_line, expected_line = output_list_lines[line2], f'("Hello, {text_str}!")'
                if actual_line != expected_line:
                    raise RuntimeError(f"Expected {expected_line} line, but found {actual_line} (line {line2})")
                logging.info(f'"{actual_line}" found in {output_list_id} at position {line2}')

            # All good!
            last_err = None
            logging.info(f"{output_list_id} lines are defined correctly")
            break
        except RuntimeError as run_err:
            last_err = str(run_err)
            logging.warning(f"Fallback hit! Sleeping for {fallback_sleep_sec} before continuing")
            time.sleep(fallback_sleep_sec)

    if last_err:
        raise RuntimeError(last_err)


def _handle_route_for_webkit(route):
    url = route.request.url.replace("https://", "http://")

    headers = None
    if any(map(url.endswith, [".css", ".js", ".svg"])):
        global _FRONTEND_URL
        assert _FRONTEND_URL
        headers = {
            "referer": _FRONTEND_URL,
        }

    response = route.fetch(url=url, headers=headers)
    assert response.status == 200, f"Expected 200 status code, but got {response.status}. Url: {url}"
    route.fulfill(response=response)


def _test_ui(ui, url, handler, browsers):
    logging.info(f'Testing "{str(ui)}" at "{url}"')

    has_err = False
    with sync_playwright() as playwright:
        for browser_name in browsers:
            logging.info(f'Checking "{browser_name}" browser')
            browser = _get_browser_obj(playwright, browser_name)
            if not browser:
                raise RuntimeError(f"Cannot determine browser object for browser {browser_name}")

            try:
                browser = browser.launch(headless=True)
                context = browser.new_context()
                page = context.new_page()

                # Attach a listener to the page's console events
                console_logs = []
                page.on("console", lambda msg: console_logs.append(msg))

                # Webkit forces HTTPS:
                #   - https://github.com/microsoft/playwright/issues/12975
                #   - https://stackoverflow.com/questions/46394682/safari-keeps-forcing-https-on-localhost
                if ui == _UI.FRONTEND and browser_name == _WEBKIT_BROWSER:
                    global _FRONTEND_URL
                    _FRONTEND_URL = url
                    page.route("**/*", _handle_route_for_webkit)

                page.goto(url)

                handler(page)
                _check_console_logs(console_logs)
            except Exception as e:
                logging.error(f"Error: {str(e)}")
                has_err = True
            finally:
                if context:
                    context.close()
                if browser:
                    browser.close()

    if has_err:
        sys.exit(1)


def _main():
    args = _get_argument_parser().parse_args()
    _validate_args(args)

    if args.frontend_url:
        _test_ui(_UI.FRONTEND, args.frontend_url, _test_frontend_ui_handler, args.browsers)
    if args.candid_url:
        _test_ui(_UI.CANDID, args.candid_url, _test_candid_ui_handler, args.browsers)

    logging.info("DONE!")


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
    _main()
