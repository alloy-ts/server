import { describe, expect, it } from 'vitest'
import {
  AdHoc,
  Fairing,
  FairingKind,
  createAdHocFairing,
  createFairing,
} from '../dist/index.js'

describe('Fairing NAPI Bindings', () => {
  it('should support FairingKind enum values', () => {
    expect(FairingKind.Ignite).toBe(1)
    expect(FairingKind.Liftoff).toBe(2)
    expect(FairingKind.Request).toBe(4)
    expect(FairingKind.Response).toBe(8)
    expect(FairingKind.Shutdown).toBe(16)
    expect(FairingKind.Singleton).toBe(32)
  })

  it('should construct Fairing and get info', () => {
    const kind = FairingKind.Request | FairingKind.Response
    const fairing = new Fairing('Request Logger', kind)
    const info = fairing.info()

    expect(info.name).toBe('Request Logger')
    expect(info.kind).toBe(12)
  })

  it('should create fairing via helper function', () => {
    const fairing = createFairing('Custom Fairing', FairingKind.Ignite)
    expect(fairing.info()).toEqual({
      name: 'Custom Fairing',
      kind: 1,
    })
  })

  it('should construct AdHoc fairings via factory methods', () => {
    const ignite = AdHoc.onIgnite('Ignite Handler')
    expect(ignite.info()).toEqual({
      name: 'Ignite Handler',
      kind: FairingKind.Ignite,
    })

    const liftoff = AdHoc.onLiftoff('Liftoff Handler')
    expect(liftoff.info()).toEqual({
      name: 'Liftoff Handler',
      kind: FairingKind.Liftoff,
    })

    const req = AdHoc.onRequest('Request Handler')
    expect(req.info()).toEqual({
      name: 'Request Handler',
      kind: FairingKind.Request,
    })

    const res = AdHoc.onResponse('Response Handler')
    expect(res.info()).toEqual({
      name: 'Response Handler',
      kind: FairingKind.Response,
    })

    const shutdown = AdHoc.onShutdown('Shutdown Handler')
    expect(shutdown.info()).toEqual({
      name: 'Shutdown Handler',
      kind: FairingKind.Shutdown,
    })
  })

  it('should create adhoc fairing via helper', () => {
    const adhoc = createAdHocFairing('AdHoc Test', FairingKind.Singleton)
    expect(adhoc.info()).toEqual({
      name: 'AdHoc Test',
      kind: 32,
    })
  })
})
