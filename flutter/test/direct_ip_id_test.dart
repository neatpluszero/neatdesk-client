import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/consts.dart';
import 'package:flutter_hbb/common/widgets/peer_card.dart';

// isDirectIpId must classify peers exactly as the backend's probe partitioning does
// (hbb_common::is_ipv4_str / is_ipv6_str / is_domain_port_str): only these endpoints are
// probed and labelled Reachable/Unreachable, and only these are kept out of bulk presence
// queries. A mismatch probes offscreen endpoints or mislabels peers.
void main() {
  group('isDirectIpId', () {
    test('direct IPv4/IPv6/domain:port endpoints are direct', () {
      expect(isDirectIpId('192.168.1.5'), isTrue);
      expect(isDirectIpId('192.168.1.20:21118'), isTrue); // IPv4 with port
      expect(isDirectIpId('fd00::1'), isTrue); // bare IPv6
      expect(isDirectIpId('[fd00::1]:21118'), isTrue); // bracketed IPv6 with port
      expect(isDirectIpId('nas.local:5900'), isTrue);
      expect(isDirectIpId('host.example.com:21116'), isTrue);
    });

    test('IDs, server-qualified ids, and incomplete forms are not direct', () {
      expect(isDirectIpId('123456789'), isFalse); // RustDesk ID
      expect(isDirectIpId('123456@server.example:21116'), isFalse); // server-qualified
      expect(isDirectIpId('example.com'), isFalse); // bare domain, no port
      expect(isDirectIpId('256.1.1.1'), isFalse); // invalid IPv4 octet
      expect(isDirectIpId(''), isFalse);
    });
  });

  group('probe-direct-ip-reachable option', () {
    // Off by default: an unset/empty value must decode to false, matching the backend's
    // explicit `== "Y"` check (src/client.rs). A regression here makes a fresh install look
    // enabled while probing is actually off.
    test('unset decodes to disabled; only "Y" enables', () {
      expect(option2bool(kOptionProbeDirectIpReachable, ''), isFalse);
      expect(option2bool(kOptionProbeDirectIpReachable, 'N'), isFalse);
      expect(option2bool(kOptionProbeDirectIpReachable, 'Y'), isTrue);
    });
  });
}
