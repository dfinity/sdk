import { test, expect } from "@playwright/test";

test("has title", async ({ page }) => {
  await page.goto("http://br5f7-7uaaa-aaaaa-qaaca-cai.localhost:4943/");

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/IC Hello Starter/);
});

test("image has loaded", async ({ page }) => {
  await page.goto("http://br5f7-7uaaa-aaaaa-qaaca-cai.localhost:4943/");

  // Set the attribute so we can read it
  await page.evaluate(async () => {
    await new Promise<void>((resolve) => {
      setTimeout(() => {
        document.querySelectorAll("img").forEach((img) => {
          img.setAttribute("complete", String(img.complete));
        });
        resolve();
      }, 1_000);
    });
  });

  const image = await page.getByAltText("DFINITY logo");
  await expect(await image.getAttribute("complete")).toBe("true");
});

test("has hello form", async ({ page }) => {
  await page.goto("http://br5f7-7uaaa-aaaaa-qaaca-cai.localhost:4943/");

  // Fill out the form
  const nameInput = await page.getByLabel("name");
  nameInput.fill("World");

  const submitButton = await page.locator("button[type='submit']");
  await submitButton.click();

  // Expects page to have a heading with the name of Installation.
  const greeting = await page.locator("#greeting");
  await expect(greeting).toBeVisible();
  await expect(await greeting.innerText()).toBe("Hello, World!");
});
