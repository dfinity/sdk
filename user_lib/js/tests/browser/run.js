// Loads test results from the page body

expect.extend({
  toPass(result) {
    return {
      message: () =>
        `${result.ok ? "passed" : "failed"}: ${result.id} ${result.name}
        \nexpected: ${JSON.stringify(result.expected)}
        \nactual: ${JSON.stringify(result.actual)}`,
      pass: result.ok,
    }
  }
})

beforeAll(async () => {
  await page.goto(PATH, { waitUntil: 'load' })
})

test('browser tests', async () => {
  page.on('console', msg => console.log(msg.text()))
  const bodyHandle = await page.$('body')
  const text = await page.evaluate(body => body.innerText, bodyHandle)
  const results = text.trim().split('\n\n').map(r => JSON.parse(r))
  results.forEach(r => {
    if (r.type !== 'end' && r.type !== 'test') {
      expect(r).toPass()
    }
  })
})
