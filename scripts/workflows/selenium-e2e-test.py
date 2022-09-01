#!/usr/bin/env python3
import time
import sys
from typing import Any, List

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.common.desired_capabilities import DesiredCapabilities
import chromedriver_autoinstaller
from pyvirtualdisplay import Display

display = Display(visible=0, size=(800, 800))
display.start()


class TestE2eCandidCanister:
    expected_console_log_entries = [
        {
            "level": "SEVERE",
            "message": "http://127.0.0.1:4943/api/v2/canister/aaa-aaaaa-aaa/read_state - Failed to load resource: the server responded with a status of 404 (Not Found)",
            "source": "network",
        },
        {
            "level": "WARNING",
            "message": "http://127.0.0.1:4943/index.js 1 Invalid asm.js: Unexpected token",
            "source": "javascript",
        },
        {
            "level": "WARNING",
            "message": 'http://127.0.0.1:4943/index.js 1:105942 "Expected to find result for path [object Object], but instead found nothing."',
            "source": "console-api",
        },
    ]

    def __init__(self, url: str):
        self.expected_console_log_entries[0][
            "message"
        ] = self.expected_console_log_entries[0]["message"].replace(
            "aaa-aaaaa-aaa", url.split("&id=")[-1]
        )
        self.url = url

    def test_website(self, driver: webdriver.Chrome):
        time.sleep(5)
        driver.find_element(By.ID, "greet").find_element(
            By.CLASS_NAME, "input-container"
        ).find_element(By.TAG_NAME, "input").send_keys("hello")
        driver.find_element(By.ID, "greet").find_element(
            By.CLASS_NAME, "button-container"
        ).find_element(By.XPATH, '//button[text()="Query"]').click()

        output = [
            o.text
            for o in driver.find_element(By.ID, "console").find_elements(
                By.CLASS_NAME, "output-line"
            )
        ]
        try:
            assert output == ["""› greet("hello")""", """("Hello, hello!")"""]
        except AssertionError:
            return ["text not matching input"]
        else:
            return []

    def test_console_log(self, driver: webdriver.Chrome):
        errors = []
        for log in driver.get_log("browser"):
            log.pop("timestamp", None)
            # log['message'] =
            if log not in self.expected_console_log_entries:
                errors.append(log)
        return errors

    def test(self, driver: webdriver.Chrome) -> List[Any]:
        driver.get(self.url)
        time.sleep(8)
        print(driver.page_source)
        errors = self.test_website(driver) + self.test_console_log(driver)
        # driver.close()
        return errors


class TestE2eFrontendCanister:
    expected_console_log_entries = []

    def __init__(self, url: str):
        self.url = url

    def test_website(self, driver: webdriver.Chrome):
        driver.find_element(By.ID, "name").send_keys("hello")
        driver.find_element(By.TAG_NAME, "button").click()
        time.sleep(2)
        try:
            assert driver.find_element(By.ID, "greeting").text == "Hello, hello!"
        except AssertionError:
            return ["text not matching input"]
        else:
            return []

    def test_console_log(self, driver: webdriver.Chrome):
        errors = []
        for log in driver.get_log("browser"):
            log.pop("timestamp", None)
            if log not in self.expected_console_log_entries:
                errors.append(log)
        return errors

    def test(self, driver: webdriver.Chrome) -> List[Any]:
        driver.get(self.url)
        time.sleep(8)
        print(driver.page_source)
        errors = self.test_website(driver) + self.test_console_log(driver)
        # driver.close()
        return errors


def main():
    def prepare_driver():
        # Check if the current version of chromedriver exists
        # and if it doesn't exist, download it automatically,
        # then add chromedriver to path
        chromedriver_autoinstaller.install()

        chrome_options = webdriver.ChromeOptions()
        options = [
            # Define window size here
            "--window-size=1200,1200",
            "--ignore-certificate-errors"
            # "--headless",
            # "--disable-gpu",
            # "--window-size=1920,1200",
            # "--ignore-certificate-errors",
            # "--disable-extensions",
            # "--no-sandbox",
            # "--disable-dev-shm-usage",
            #'--remote-debugging-port=9222'
        ]
        for option in options:
            chrome_options.add_argument(option)

        # enable browser logging
        capabilities = webdriver.DesiredCapabilities.CHROME.copy()
        capabilities["goog:loggingPrefs"] = {"browser": "INFO"}
        chrome_options.add_experimental_option("excludeSwitches", ["enable-logging"])
        driver = webdriver.Chrome(
            desired_capabilities=capabilities, options=chrome_options
        )

        return driver

    _, frontend_url, candid_ui_url = sys.argv
    print(f"{frontend_url = }")
    print(f"{candid_ui_url = }")

    driver = prepare_driver()
    print(f"initialized {driver = }")

    frontend_errors = []
    candid_errors = []

    # run the tests several times
    try:
        for _ in range(3):
            frontend_errors.append(TestE2eFrontendCanister(frontend_url).test(driver))
        for _ in range(3):
            candid_errors.append(TestE2eCandidCanister(candid_ui_url).test(driver))
    except Exception as e:
        print(f"Exception: {e}")
        raise e

    raise_me = False

    try:
        assert not candid_errors
    except AssertionError as e:
        print(f"{candid_errors = }")
        raise_me = True
    try:
        assert not frontend_errors
    except AssertionError as e:
        print(f"{frontend_errors = }")
        raise_me = True

    if raise_me:
        raise AssertionError("There were errors")


if __name__ == "__main__":
    main()
