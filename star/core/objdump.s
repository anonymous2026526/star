=== A=0x401C5A4 F=0x1c5a4 ===

./target/release/analysis:     file format elf64-x86-64


Disassembly of section .text:

000000000001c3a4 <<hmac::simple::SimpleHmac<D> as digest::FixedOutput>::finalize_into+0x314>:
   1c3a4:	0f 29 85 60 ff ff ff 	movaps %xmm0,-0xa0(%rbp)
   1c3ab:	4c 8d 75 88          	lea    -0x78(%rbp),%r14
   1c3af:	44 0f b6 7d c8       	movzbl -0x38(%rbp),%r15d
   1c3b4:	48 8b 45 80          	mov    -0x80(%rbp),%rax
   1c3b8:	48 c1 e0 09          	shl    $0x9,%rax
   1c3bc:	46 8d 24 fd 00 00 00 	lea    0x0(,%r15,8),%r12d
   1c3c3:	00 
   1c3c4:	49 09 c4             	or     %rax,%r12
   1c3c7:	49 0f cc             	bswap  %r12
   1c3ca:	42 c6 44 3d 88 80    	movb   $0x80,-0x78(%rbp,%r15,1)
   1c3d0:	41 83 ff 3f          	cmp    $0x3f,%r15d
   1c3d4:	74 24                	je     1c3fa <<hmac::simple::SimpleHmac<D> as digest::FixedOutput>::finalize_into+0x36a>
   1c3d6:	49 8d 3c 2f          	lea    (%r15,%rbp,1),%rdi
   1c3da:	48 83 c7 88          	add    $0xffffffffffffff88,%rdi
   1c3de:	48 ff c7             	inc    %rdi
   1c3e1:	4c 89 fa             	mov    %r15,%rdx
   1c3e4:	48 83 f2 3f          	xor    $0x3f,%rdx
   1c3e8:	31 f6                	xor    %esi,%esi
   1c3ea:	ff 15 e8 41 07 00    	call   *0x741e8(%rip)        # 905d8 <memset@GLIBC_2.2.5>
   1c3f0:	41 83 f7 38          	xor    $0x38,%r15d
   1c3f4:	41 83 ff 07          	cmp    $0x7,%r15d
   1c3f8:	77 5a                	ja     1c454 <<hmac::simple::SimpleHmac<D> as digest::FixedOutput>::finalize_into+0x3c4>
   1c3fa:	4c 8d 2d 5f 6c 00 00 	lea    0x6c5f(%rip),%r13        # 23060 <sha2::sha256::compress256>
   1c401:	4c 8d bd 60 ff ff ff 	lea    -0xa0(%rbp),%r15
   1c408:	ba 01 00 00 00       	mov    $0x1,%edx
   1c40d:	4c 89 ff             	mov    %r15,%rdi
   1c410:	4c 89 f6             	mov    %r14,%rsi
   1c413:	41 ff d5             	call   *%r13
   1c416:	0f 57 c0             	xorps  %xmm0,%xmm0
   1c419:	0f 29 85 d0 fe ff ff 	movaps %xmm0,-0x130(%rbp)
   1c420:	0f 29 85 c0 fe ff ff 	movaps %xmm0,-0x140(%rbp)
   1c427:	0f 29 85 b0 fe ff ff 	movaps %xmm0,-0x150(%rbp)
   1c42e:	48 c7 85 e0 fe ff ff 	movq   $0x0,-0x120(%rbp)
   1c435:	00 00 00 00 
   1c439:	4c 89 a5 e8 fe ff ff 	mov    %r12,-0x118(%rbp)
   1c440:	48 8d b5 b0 fe ff ff 	lea    -0x150(%rbp),%rsi
   1c447:	ba 01 00 00 00       	mov    $0x1,%edx
   1c44c:	4c 89 ff             	mov    %r15,%rdi
   1c44f:	41 ff d5             	call   *%r13
   1c452:	eb 19                	jmp    1c46d <<hmac::simple::SimpleHmac<D> as digest::FixedOutput>::finalize_into+0x3dd>
   1c454:	4c 89 65 c0          	mov    %r12,-0x40(%rbp)
   1c458:	48 8d bd 60 ff ff ff 	lea    -0xa0(%rbp),%rdi
   1c45f:	ba 01 00 00 00       	mov    $0x1,%edx
   1c464:	4c 89 f6             	mov    %r14,%rsi
   1c467:	ff 15 eb 4a 07 00    	call   *0x74aeb(%rip)        # 90f58 <_GLOBAL_OFFSET_TABLE_+0xa48>
   1c46d:	66 0f 6f 85 60 ff ff 	movdqa -0xa0(%rbp),%xmm0
   1c474:	ff 
   1c475:	66 0f ef c9          	pxor   %xmm1,%xmm1
   1c479:	66 0f 6f d0          	movdqa %xmm0,%xmm2
   1c47d:	66 0f 68 d1          	punpckhbw %xmm1,%xmm2
   1c481:	f2 0f 70 d2 1b       	pshuflw $0x1b,%xmm2,%xmm2
   1c486:	f3 0f 70 d2 1b       	pshufhw $0x1b,%xmm2,%xmm2
   1c48b:	66 0f 60 c1          	punpcklbw %xmm1,%xmm0
   1c48f:	f2 0f 70 c0 1b       	pshuflw $0x1b,%xmm0,%xmm0
   1c494:	f3 0f 70 c0 1b       	pshufhw $0x1b,%xmm0,%xmm0
   1c499:	66 0f 67 c2          	packuswb %xmm2,%xmm0
   1c49d:	f3 0f 7f 03          	movdqu %xmm0,(%rbx)
   1c4a1:	66 0f 6f 85 70 ff ff 	movdqa -0x90(%rbp),%xmm0
   1c4a8:	ff 
   1c4a9:	66 0f 6f d0          	movdqa %xmm0,%xmm2
   1c4ad:	66 0f 68 d1          	punpckhbw %xmm1,%xmm2
   1c4b1:	f2 0f 70 d2 1b       	pshuflw $0x1b,%xmm2,%xmm2
   1c4b6:	f3 0f 70 d2 1b       	pshufhw $0x1b,%xmm2,%xmm2
   1c4bb:	66 0f 60 c1          	punpcklbw %xmm1,%xmm0
   1c4bf:	f2 0f 70 c0 1b       	pshuflw $0x1b,%xmm0,%xmm0
   1c4c4:	f3 0f 70 c0 1b       	pshufhw $0x1b,%xmm0,%xmm0
   1c4c9:	66 0f 67 c2          	packuswb %xmm2,%xmm0
   1c4cd:	f3 0f 7f 43 10       	movdqu %xmm0,0x10(%rbx)
   1c4d2:	48 81 c4 28 01 00 00 	add    $0x128,%rsp
   1c4d9:	5b                   	pop    %rbx
   1c4da:	41 5c                	pop    %r12
   1c4dc:	41 5d                	pop    %r13
   1c4de:	41 5e                	pop    %r14
   1c4e0:	41 5f                	pop    %r15
   1c4e2:	5d                   	pop    %rbp
   1c4e3:	c3                   	ret
   1c4e4:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
   1c4eb:	00 00 00 
   1c4ee:	66 90                	xchg   %ax,%ax

000000000001c4f0 <<hpke::kem::dhkem::x25519_hkdfsha256::EncappedKey as hpke::Serializable>::to_bytes>:
   1c4f0:	55                   	push   %rbp
   1c4f1:	48 89 e5             	mov    %rsp,%rbp
   1c4f4:	53                   	push   %rbx
   1c4f5:	50                   	push   %rax
   1c4f6:	48 89 fb             	mov    %rdi,%rbx
   1c4f9:	ff 15 89 4a 07 00    	call   *0x74a89(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   1c4ff:	48 89 d8             	mov    %rbx,%rax
   1c502:	48 83 c4 08          	add    $0x8,%rsp
   1c506:	5b                   	pop    %rbx
   1c507:	5d                   	pop    %rbp
   1c508:	c3                   	ret
   1c509:	0f 1f 80 00 00 00 00 	nopl   0x0(%rax)

000000000001c510 <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::derive_keypair>:
   1c510:	55                   	push   %rbp
   1c511:	48 89 e5             	mov    %rsp,%rbp
   1c514:	53                   	push   %rbx
   1c515:	50                   	push   %rax
   1c516:	48 89 d1             	mov    %rdx,%rcx
   1c519:	48 89 f2             	mov    %rsi,%rdx
   1c51c:	48 89 fb             	mov    %rdi,%rbx
   1c51f:	c6 45 f4 4d          	movb   $0x4d,-0xc(%rbp)
   1c523:	66 c7 45 f2 4b 45    	movw   $0x454b,-0xe(%rbp)
   1c529:	66 c7 45 f5 00 20    	movw   $0x2000,-0xb(%rbp)
   1c52f:	48 8d 75 f2          	lea    -0xe(%rbp),%rsi
   1c533:	e8 28 e5 ff ff       	call   1aa60 <<hpke::dhkex::x25519::X25519 as hpke::dhkex::DhKeyExchange>::derive_keypair>
   1c538:	48 89 d8             	mov    %rbx,%rax
   1c53b:	48 83 c4 08          	add    $0x8,%rsp
   1c53f:	5b                   	pop    %rbx
   1c540:	5d                   	pop    %rbp
   1c541:	c3                   	ret
   1c542:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
   1c549:	00 00 00 
   1c54c:	0f 1f 40 00          	nopl   0x0(%rax)

000000000001c550 <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap>:
   1c550:	55                   	push   %rbp
   1c551:	48 89 e5             	mov    %rsp,%rbp
   1c554:	41 57                	push   %r15
   1c556:	41 56                	push   %r14
   1c558:	41 55                	push   %r13
   1c55a:	41 54                	push   %r12
   1c55c:	53                   	push   %rbx
   1c55d:	48 81 ec 68 05 00 00 	sub    $0x568,%rsp
   1c564:	49 89 cc             	mov    %rcx,%r12
   1c567:	49 89 d6             	mov    %rdx,%r14
   1c56a:	49 89 f7             	mov    %rsi,%r15
   1c56d:	48 89 fb             	mov    %rdi,%rbx
   1c570:	c6 45 d4 4d          	movb   $0x4d,-0x2c(%rbp)
   1c574:	66 c7 45 d2 4b 45    	movw   $0x454b,-0x2e(%rbp)
   1c57a:	66 c7 45 d5 00 20    	movw   $0x2000,-0x2b(%rbp)
   1c580:	4c 8d ad e0 fd ff ff 	lea    -0x220(%rbp),%r13
   1c587:	4c 89 ef             	mov    %r13,%rdi
   1c58a:	48 89 ca             	mov    %rcx,%rdx
   1c58d:	ff 15 e5 42 07 00    	call   *0x742e5(%rip)        # 90878 <_GLOBAL_OFFSET_TABLE_+0x368>
   1c593:	48 8d 35 b1 58 05 00 	lea    0x558b1(%rip),%rsi        # 71e4b <anon.39a2a9f1401edcf2cd24a86c78a26440.25.llvm.12169493792077094199>
   1c59a:	4c 89 ef             	mov    %r13,%rdi
   1c59d:	e8 ce e0 ff ff       	call   1a670 <<[T] as subtle::ConstantTimeEq>::ct_eq>
   1c5a2:	84 c0                	test   %al,%al
   1c5a4:	74 1d                	je     1c5c3 <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap+0x73>
   1c5a6:	48 8d bd e0 fd ff ff 	lea    -0x220(%rbp),%rdi
   1c5ad:	ff 15 0d 4a 07 00    	call   *0x74a0d(%rip)        # 90fc0 <_GLOBAL_OFFSET_TABLE_+0xab0>
   1c5b3:	48 c7 43 08 06 00 00 	movq   $0x6,0x8(%rbx)
   1c5ba:	00 
   1c5bb:	c6 03 01             	movb   $0x1,(%rbx)
   1c5be:	e9 9a 04 00 00       	jmp    1ca5d <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap+0x50d>
   1c5c3:	8b 85 e0 fd ff ff    	mov    -0x220(%rbp),%eax
   1c5c9:	8b 8d e3 fd ff ff    	mov    -0x21d(%rbp),%ecx
   1c5cf:	89 4d 93             	mov    %ecx,-0x6d(%rbp)
   1c5d2:	89 45 90             	mov    %eax,-0x70(%rbp)
   1c5d5:	48 8b 85 e7 fd ff ff 	mov    -0x219(%rbp),%rax
   1c5dc:	0f 10 85 ef fd ff ff 	movups -0x211(%rbp),%xmm0
   1c5e3:	0f 11 45 9f          	movups %xmm0,-0x61(%rbp)
   1c5e7:	0f b6 8d ff fd ff ff 	movzbl -0x201(%rbp),%ecx
   1c5ee:	48 89 45 97          	mov    %rax,-0x69(%rbp)
   1c5f2:	88 4d af             	mov    %cl,-0x51(%rbp)
   1c5f5:	48 8d bd c0 fd ff ff 	lea    -0x240(%rbp),%rdi
   1c5fc:	4c 89 fe             	mov    %r15,%rsi
   1c5ff:	ff 15 13 43 07 00    	call   *0x74313(%rip)        # 90918 <_GLOBAL_OFFSET_TABLE_+0x408>
   1c605:	4d 85 f6             	test   %r14,%r14
   1c608:	0f 84 00 01 00 00    	je     1c70e <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap+0x1be>
   1c60e:	48 8d bd 40 fe ff ff 	lea    -0x1c0(%rbp),%rdi
   1c615:	ba 2f 01 00 00       	mov    $0x12f,%edx
   1c61a:	31 f6                	xor    %esi,%esi
   1c61c:	ff 15 b6 3f 07 00    	call   *0x73fb6(%rip)        # 905d8 <memset@GLIBC_2.2.5>
   1c622:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c629:	4c 89 e6             	mov    %r12,%rsi
   1c62c:	ff 15 56 49 07 00    	call   *0x74956(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   1c632:	0f 10 85 31 fc ff ff 	movups -0x3cf(%rbp),%xmm0
   1c639:	0f 10 8d 41 fc ff ff 	movups -0x3bf(%rbp),%xmm1
   1c640:	0f 29 8d f0 fd ff ff 	movaps %xmm1,-0x210(%rbp)
   1c647:	0f 29 85 e0 fd ff ff 	movaps %xmm0,-0x220(%rbp)
   1c64e:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c655:	48 8d b5 c0 fd ff ff 	lea    -0x240(%rbp),%rsi
   1c65c:	ff 15 26 49 07 00    	call   *0x74926(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   1c662:	0f 10 85 31 fc ff ff 	movups -0x3cf(%rbp),%xmm0
   1c669:	0f 10 8d 41 fc ff ff 	movups -0x3bf(%rbp),%xmm1
   1c670:	0f 29 8d 10 fe ff ff 	movaps %xmm1,-0x1f0(%rbp)
   1c677:	0f 29 85 00 fe ff ff 	movaps %xmm0,-0x200(%rbp)
   1c67e:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c685:	4c 89 f6             	mov    %r14,%rsi
   1c688:	ff 15 fa 48 07 00    	call   *0x748fa(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   1c68e:	0f 10 85 31 fc ff ff 	movups -0x3cf(%rbp),%xmm0
   1c695:	0f 10 8d 41 fc ff ff 	movups -0x3bf(%rbp),%xmm1
   1c69c:	0f 29 8d 30 fe ff ff 	movaps %xmm1,-0x1d0(%rbp)
   1c6a3:	0f 29 85 20 fe ff ff 	movaps %xmm0,-0x1e0(%rbp)
   1c6aa:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c6b1:	48 8d b5 e0 fd ff ff 	lea    -0x220(%rbp),%rsi
   1c6b8:	ba 8f 01 00 00       	mov    $0x18f,%edx
   1c6bd:	ff 15 b5 48 07 00    	call   *0x748b5(%rip)        # 90f78 <memcpy@GLIBC_2.14>
   1c6c3:	48 8d bd e0 fd ff ff 	lea    -0x220(%rbp),%rdi
   1c6ca:	4c 89 fe             	mov    %r15,%rsi
   1c6cd:	4c 89 f2             	mov    %r14,%rdx
   1c6d0:	ff 15 a2 41 07 00    	call   *0x741a2(%rip)        # 90878 <_GLOBAL_OFFSET_TABLE_+0x368>
   1c6d6:	48 8d 35 6e 57 05 00 	lea    0x5576e(%rip),%rsi        # 71e4b <anon.39a2a9f1401edcf2cd24a86c78a26440.25.llvm.12169493792077094199>
   1c6dd:	48 8d bd e0 fd ff ff 	lea    -0x220(%rbp),%rdi
   1c6e4:	e8 87 df ff ff       	call   1a670 <<[T] as subtle::ConstantTimeEq>::ct_eq>
   1c6e9:	84 c0                	test   %al,%al
   1c6eb:	0f 84 a7 01 00 00    	je     1c898 <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap+0x348>
   1c6f1:	48 8d bd e0 fd ff ff 	lea    -0x220(%rbp),%rdi
   1c6f8:	ff 15 c2 48 07 00    	call   *0x748c2(%rip)        # 90fc0 <_GLOBAL_OFFSET_TABLE_+0xab0>
   1c6fe:	48 c7 43 08 06 00 00 	movq   $0x6,0x8(%rbx)
   1c705:	00 
   1c706:	c6 03 01             	movb   $0x1,(%rbx)
   1c709:	e9 45 03 00 00       	jmp    1ca53 <<hpke::kem::dhkem::x25519_hkdfsha256::X25519HkdfSha256 as hpke::kem::Kem>::decap+0x503>
   1c70e:	0f 57 c0             	xorps  %xmm0,%xmm0
   1c711:	0f 11 85 da fe ff ff 	movups %xmm0,-0x126(%rbp)
   1c718:	0f 11 85 d0 fe ff ff 	movups %xmm0,-0x130(%rbp)
   1c71f:	0f 11 85 c0 fe ff ff 	movups %xmm0,-0x140(%rbp)
   1c726:	0f 11 85 b0 fe ff ff 	movups %xmm0,-0x150(%rbp)
   1c72d:	0f 11 85 a0 fe ff ff 	movups %xmm0,-0x160(%rbp)
   1c734:	0f 11 85 90 fe ff ff 	movups %xmm0,-0x170(%rbp)
   1c73b:	0f 11 85 80 fe ff ff 	movups %xmm0,-0x180(%rbp)
   1c742:	0f 11 85 70 fe ff ff 	movups %xmm0,-0x190(%rbp)
   1c749:	0f 11 85 60 fe ff ff 	movups %xmm0,-0x1a0(%rbp)
   1c750:	0f 11 85 50 fe ff ff 	movups %xmm0,-0x1b0(%rbp)
   1c757:	0f 11 85 40 fe ff ff 	movups %xmm0,-0x1c0(%rbp)
   1c75e:	0f 11 85 30 fe ff ff 	movups %xmm0,-0x1d0(%rbp)
   1c765:	0f 11 85 20 fe ff ff 	movups %xmm0,-0x1e0(%rbp)
   1c76c:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c773:	4c 89 e6             	mov    %r12,%rsi
   1c776:	ff 15 0c 48 07 00    	call   *0x7480c(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   1c77c:	0f 10 85 31 fc ff ff 	movups -0x3cf(%rbp),%xmm0
   1c783:	0f 10 8d 41 fc ff ff 	movups -0x3bf(%rbp),%xmm1
   1c78a:	0f 29 8d f0 fd ff ff 	movaps %xmm1,-0x210(%rbp)
   1c791:	0f 29 85 e0 fd ff ff 	movaps %xmm0,-0x220(%rbp)
   1c798:	48 8d bd 31 fc ff ff 	lea    -0x3cf(%rbp),%rdi
   1c79f:	48                   	rex.W
   1c7a0:	8d                   	.byte 0x8d
   1c7a1:	b5 c0                	mov    $0xc0,%ch
   1c7a3:	fd                   	std
=== A=0x4017CD5 F=0x17cd5 ===

./target/release/analysis:     file format elf64-x86-64


Disassembly of section .text:

0000000000017ad5 <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x315>:
   17ad5:	f6 ff                	idiv   %bh
   17ad7:	15 94 8a 07 00       	adc    $0x78a94,%eax
   17adc:	0f b6 44 24 20       	movzbl 0x20(%rsp),%eax
   17ae1:	31 ff                	xor    %edi,%edi
   17ae3:	4c 8b 64 24 38       	mov    0x38(%rsp),%r12
   17ae8:	41 3a 04 24          	cmp    (%r12),%al
   17aec:	40 0f 94 c7          	sete   %dil
   17af0:	ff 15 c2 8b 07 00    	call   *0x78bc2(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17af6:	88 44 24 10          	mov    %al,0x10(%rsp)
   17afa:	0f b6 44 24 21       	movzbl 0x21(%rsp),%eax
   17aff:	31 ff                	xor    %edi,%edi
   17b01:	41 3a 44 24 01       	cmp    0x1(%r12),%al
   17b06:	40 0f 94 c7          	sete   %dil
   17b0a:	ff 15 a8 8b 07 00    	call   *0x78ba8(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b10:	88 44 24 0f          	mov    %al,0xf(%rsp)
   17b14:	0f b6 44 24 22       	movzbl 0x22(%rsp),%eax
   17b19:	31 ff                	xor    %edi,%edi
   17b1b:	41 3a 44 24 02       	cmp    0x2(%r12),%al
   17b20:	40 0f 94 c7          	sete   %dil
   17b24:	ff 15 8e 8b 07 00    	call   *0x78b8e(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b2a:	88 44 24 0e          	mov    %al,0xe(%rsp)
   17b2e:	0f b6 44 24 23       	movzbl 0x23(%rsp),%eax
   17b33:	31 ff                	xor    %edi,%edi
   17b35:	41 3a 44 24 03       	cmp    0x3(%r12),%al
   17b3a:	40 0f 94 c7          	sete   %dil
   17b3e:	ff 15 74 8b 07 00    	call   *0x78b74(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b44:	88 44 24 0d          	mov    %al,0xd(%rsp)
   17b48:	0f b6 44 24 24       	movzbl 0x24(%rsp),%eax
   17b4d:	31 ff                	xor    %edi,%edi
   17b4f:	41 3a 44 24 04       	cmp    0x4(%r12),%al
   17b54:	40 0f 94 c7          	sete   %dil
   17b58:	ff 15 5a 8b 07 00    	call   *0x78b5a(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b5e:	41 89 c6             	mov    %eax,%r14d
   17b61:	0f b6 44 24 25       	movzbl 0x25(%rsp),%eax
   17b66:	31 ff                	xor    %edi,%edi
   17b68:	41 3a 44 24 05       	cmp    0x5(%r12),%al
   17b6d:	40 0f 94 c7          	sete   %dil
   17b71:	ff 15 41 8b 07 00    	call   *0x78b41(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b77:	88 44 24 0c          	mov    %al,0xc(%rsp)
   17b7b:	0f b6 44 24 26       	movzbl 0x26(%rsp),%eax
   17b80:	31 ff                	xor    %edi,%edi
   17b82:	41 3a 44 24 06       	cmp    0x6(%r12),%al
   17b87:	40 0f 94 c7          	sete   %dil
   17b8b:	ff 15 27 8b 07 00    	call   *0x78b27(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17b91:	88 44 24 0b          	mov    %al,0xb(%rsp)
   17b95:	0f b6 44 24 27       	movzbl 0x27(%rsp),%eax
   17b9a:	31 ff                	xor    %edi,%edi
   17b9c:	41 3a 44 24 07       	cmp    0x7(%r12),%al
   17ba1:	40 0f 94 c7          	sete   %dil
   17ba5:	ff 15 0d 8b 07 00    	call   *0x78b0d(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17bab:	41 89 c7             	mov    %eax,%r15d
   17bae:	0f b6 44 24 28       	movzbl 0x28(%rsp),%eax
   17bb3:	31 ff                	xor    %edi,%edi
   17bb5:	41 3a 44 24 08       	cmp    0x8(%r12),%al
   17bba:	40 0f 94 c7          	sete   %dil
   17bbe:	ff 15 f4 8a 07 00    	call   *0x78af4(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17bc4:	88 44 24 0a          	mov    %al,0xa(%rsp)
   17bc8:	0f b6 44 24 29       	movzbl 0x29(%rsp),%eax
   17bcd:	31 ff                	xor    %edi,%edi
   17bcf:	41 3a 44 24 09       	cmp    0x9(%r12),%al
   17bd4:	40 0f 94 c7          	sete   %dil
   17bd8:	ff 15 da 8a 07 00    	call   *0x78ada(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17bde:	88 44 24 09          	mov    %al,0x9(%rsp)
   17be2:	0f b6 44 24 2a       	movzbl 0x2a(%rsp),%eax
   17be7:	31 ff                	xor    %edi,%edi
   17be9:	41 3a 44 24 0a       	cmp    0xa(%r12),%al
   17bee:	40 0f 94 c7          	sete   %dil
   17bf2:	ff 15 c0 8a 07 00    	call   *0x78ac0(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17bf8:	88 44 24 08          	mov    %al,0x8(%rsp)
   17bfc:	0f b6 44 24 2b       	movzbl 0x2b(%rsp),%eax
   17c01:	31 ff                	xor    %edi,%edi
   17c03:	41 3a 44 24 0b       	cmp    0xb(%r12),%al
   17c08:	40 0f 94 c7          	sete   %dil
   17c0c:	ff 15 a6 8a 07 00    	call   *0x78aa6(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17c12:	41 89 c5             	mov    %eax,%r13d
   17c15:	0f b6 44 24 2c       	movzbl 0x2c(%rsp),%eax
   17c1a:	31 ff                	xor    %edi,%edi
   17c1c:	41 3a 44 24 0c       	cmp    0xc(%r12),%al
   17c21:	40 0f 94 c7          	sete   %dil
   17c25:	ff 15 8d 8a 07 00    	call   *0x78a8d(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17c2b:	88 44 24 07          	mov    %al,0x7(%rsp)
   17c2f:	0f b6 44 24 2d       	movzbl 0x2d(%rsp),%eax
   17c34:	31 ff                	xor    %edi,%edi
   17c36:	41 3a 44 24 0d       	cmp    0xd(%r12),%al
   17c3b:	40 0f 94 c7          	sete   %dil
   17c3f:	ff 15 73 8a 07 00    	call   *0x78a73(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17c45:	88 44 24 06          	mov    %al,0x6(%rsp)
   17c49:	0f b6 44 24 2e       	movzbl 0x2e(%rsp),%eax
   17c4e:	31 ff                	xor    %edi,%edi
   17c50:	41 3a 44 24 0e       	cmp    0xe(%r12),%al
   17c55:	40 0f 94 c7          	sete   %dil
   17c59:	ff 15 59 8a 07 00    	call   *0x78a59(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17c5f:	88 44 24 05          	mov    %al,0x5(%rsp)
   17c63:	0f b6 44 24 2f       	movzbl 0x2f(%rsp),%eax
   17c68:	31 ff                	xor    %edi,%edi
   17c6a:	41 3a 44 24 0f       	cmp    0xf(%r12),%al
   17c6f:	40 0f 94 c7          	sete   %dil
   17c73:	ff 15 3f 8a 07 00    	call   *0x78a3f(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17c79:	0f b6 4c 24 10       	movzbl 0x10(%rsp),%ecx
   17c7e:	22 4c 24 0f          	and    0xf(%rsp),%cl
   17c82:	0f b6 54 24 0e       	movzbl 0xe(%rsp),%edx
   17c87:	22 54 24 0d          	and    0xd(%rsp),%dl
   17c8b:	20 ca                	and    %cl,%dl
   17c8d:	44 22 74 24 0c       	and    0xc(%rsp),%r14b
   17c92:	44 22 74 24 0b       	and    0xb(%rsp),%r14b
   17c97:	41 20 d6             	and    %dl,%r14b
   17c9a:	44 22 7c 24 0a       	and    0xa(%rsp),%r15b
   17c9f:	44 22 7c 24 09       	and    0x9(%rsp),%r15b
   17ca4:	44 22 7c 24 08       	and    0x8(%rsp),%r15b
   17ca9:	45 20 f7             	and    %r14b,%r15b
   17cac:	44 22 6c 24 07       	and    0x7(%rsp),%r13b
   17cb1:	44 22 6c 24 06       	and    0x6(%rsp),%r13b
   17cb6:	44 22 6c 24 05       	and    0x5(%rsp),%r13b
   17cbb:	41 20 c5             	and    %al,%r13b
   17cbe:	45 20 fd             	and    %r15b,%r13b
   17cc1:	41 80 e5 01          	and    $0x1,%r13b
   17cc5:	41 0f b6 fd          	movzbl %r13b,%edi
   17cc9:	ff 15 e9 89 07 00    	call   *0x789e9(%rip)        # 906b8 <_GLOBAL_OFFSET_TABLE_+0x1a8>
   17ccf:	84 c0                	test   %al,%al
   17cd1:	41 0f 94 c6          	sete   %r14b
   17cd5:	74 16                	je     17ced <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x52d>
   17cd7:	48 8d bb 40 02 00 00 	lea    0x240(%rbx),%rdi
   17cde:	48 8b 74 24 30       	mov    0x30(%rsp),%rsi
   17ce3:	48 8b 54 24 18       	mov    0x18(%rsp),%rdx
   17ce8:	e8 13 04 00 00       	call   18100 <cipher::stream::StreamCipher::apply_keystream>
   17ced:	4c 8d bb 40 02 00 00 	lea    0x240(%rbx),%r15
   17cf4:	4c 89 ff             	mov    %r15,%rdi
   17cf7:	e8 94 d3 ff ff       	call   15090 <<cipher::stream_wrapper::StreamCipherCoreWrapper<T> as core::ops::drop::Drop>::drop>
   17cfc:	41 c7 07 00 00 00 00 	movl   $0x0,(%r15)
   17d03:	c7 83 44 02 00 00 00 	movl   $0x0,0x244(%rbx)
   17d0a:	00 00 00 
   17d0d:	c7 83 48 02 00 00 00 	movl   $0x0,0x248(%rbx)
   17d14:	00 00 00 
   17d17:	c7 83 4c 02 00 00 00 	movl   $0x0,0x24c(%rbx)
   17d1e:	00 00 00 
   17d21:	c7 83 50 02 00 00 00 	movl   $0x0,0x250(%rbx)
   17d28:	00 00 00 
   17d2b:	c7 83 54 02 00 00 00 	movl   $0x0,0x254(%rbx)
   17d32:	00 00 00 
   17d35:	c7 83 58 02 00 00 00 	movl   $0x0,0x258(%rbx)
   17d3c:	00 00 00 
   17d3f:	c7 83 5c 02 00 00 00 	movl   $0x0,0x25c(%rbx)
   17d46:	00 00 00 
   17d49:	c7 83 60 02 00 00 00 	movl   $0x0,0x260(%rbx)
   17d50:	00 00 00 
   17d53:	c7 83 64 02 00 00 00 	movl   $0x0,0x264(%rbx)
   17d5a:	00 00 00 
   17d5d:	c7 83 68 02 00 00 00 	movl   $0x0,0x268(%rbx)
   17d64:	00 00 00 
   17d67:	c7 83 6c 02 00 00 00 	movl   $0x0,0x26c(%rbx)
   17d6e:	00 00 00 
   17d71:	c7 83 70 02 00 00 00 	movl   $0x0,0x270(%rbx)
   17d78:	00 00 00 
   17d7b:	c7 83 74 02 00 00 00 	movl   $0x0,0x274(%rbx)
   17d82:	00 00 00 
   17d85:	c7 83 78 02 00 00 00 	movl   $0x0,0x278(%rbx)
   17d8c:	00 00 00 
   17d8f:	c7 83 7c 02 00 00 00 	movl   $0x0,0x27c(%rbx)
   17d96:	00 00 00 
   17d99:	44 89 f0             	mov    %r14d,%eax
   17d9c:	48 8d 65 d8          	lea    -0x28(%rbp),%rsp
   17da0:	5b                   	pop    %rbx
   17da1:	41 5c                	pop    %r12
   17da3:	41 5d                	pop    %r13
   17da5:	41 5e                	pop    %r14
   17da7:	41 5f                	pop    %r15
   17da9:	5d                   	pop    %rbp
   17daa:	c3                   	ret
   17dab:	eb 3e                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17dad:	eb 3c                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17daf:	eb 3a                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17db1:	eb 38                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17db3:	eb 36                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17db5:	eb 34                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17db7:	49 89 c6             	mov    %rax,%r14
   17dba:	4c 89 ff             	mov    %r15,%rdi
   17dbd:	e8 ee 17 00 00       	call   195b0 <<chacha20::ChaChaCore<R> as core::ops::drop::Drop>::drop>
   17dc2:	eb 32                	jmp    17df6 <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x636>
   17dc4:	ff 15 2e 91 07 00    	call   *0x7912e(%rip)        # 90ef8 <_GLOBAL_OFFSET_TABLE_+0x9e8>
   17dca:	49 89 c6             	mov    %rax,%r14
   17dcd:	4c 89 ff             	mov    %r15,%rdi
   17dd0:	e8 db 17 00 00       	call   195b0 <<chacha20::ChaChaCore<R> as core::ops::drop::Drop>::drop>
   17dd5:	eb 1f                	jmp    17df6 <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x636>
   17dd7:	ff 15 1b 91 07 00    	call   *0x7911b(%rip)        # 90ef8 <_GLOBAL_OFFSET_TABLE_+0x9e8>
   17ddd:	eb 0c                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17ddf:	eb 0a                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17de1:	eb 08                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17de3:	eb 06                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17de5:	eb 04                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17de7:	eb 02                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17de9:	eb 00                	jmp    17deb <chacha20poly1305::cipher::Cipher<C>::decrypt_in_place_detached+0x62b>
   17deb:	49 89 c6             	mov    %rax,%r14
   17dee:	48 89 df             	mov    %rbx,%rdi
   17df1:	e8 2a 02 00 00       	call   18020 <core::ptr::drop_in_place<chacha20poly1305::cipher::Cipher<cipher::stream_wrapper::StreamCipherCoreWrapper<chacha20::ChaChaCore<typenum::uint::UInt<typenum::uint::UInt<typenum::uint::UInt<typenum::uint::UInt<typenum::uint::UTerm,typenum::bit::B1>,typenum::bit::B0>,typenum::bit::B1>,typenum::bit::B0>>>>>>
   17df6:	4c 89 f7             	mov    %r14,%rdi
   17df9:	e8 52 02 ff ff       	call   8050 <_Unwind_Resume@plt>
   17dfe:	ff 15 f4 90 07 00    	call   *0x790f4(%rip)        # 90ef8 <_GLOBAL_OFFSET_TABLE_+0x9e8>
   17e04:	66 2e 0f 1f 84 00 00 	cs nopw 0x0(%rax,%rax,1)
   17e0b:	00 00 00 
   17e0e:	66 90                	xchg   %ax,%ax

0000000000017e10 <chacha20poly1305::cipher::Cipher<C>::new>:
   17e10:	55                   	push   %rbp
   17e11:	48 89 e5             	mov    %rsp,%rbp
   17e14:	41 57                	push   %r15
   17e16:	41 56                	push   %r14
   17e18:	53                   	push   %rbx
   17e19:	48 83 e4 e0          	and    $0xffffffffffffffe0,%rsp
   17e1d:	48 81 ec 80 02 00 00 	sub    $0x280,%rsp
   17e24:	49 89 f6             	mov    %rsi,%r14
   17e27:	48 89 fb             	mov    %rdi,%rbx
   17e2a:	0f 57 c0             	xorps  %xmm0,%xmm0
   17e2d:	0f 29 44 24 10       	movaps %xmm0,0x10(%rsp)
   17e32:	0f 29 04 24          	movaps %xmm0,(%rsp)
   17e36:	48 89 e6             	mov    %rsp,%rsi
   17e39:	ba 20 00 00 00       	mov    $0x20,%edx
   17e3e:	4c 89 f7             	mov    %r14,%rdi
   17e41:	e8 ba 02 00 00       	call   18100 <cipher::stream::StreamCipher::apply_keystream>
   17e46:	48 8d 7c 24 20       	lea    0x20(%rsp),%rdi
   17e4b:	48 89 e6             	mov    %rsp,%rsi
   17e4e:	ff 15 f4 8b 07 00    	call   *0x78bf4(%rip)        # 90a48 <_GLOBAL_OFFSET_TABLE_+0x538>
   17e54:	c6 04 24 00          	movb   $0x0,(%rsp)
   17e58:	c6 44 24 01 00       	movb   $0x0,0x1(%rsp)
   17e5d:	c6 44 24 02 00       	movb   $0x0,0x2(%rsp)
   17e62:	c6 44 24 03 00       	movb   $0x0,0x3(%rsp)
   17e67:	c6 44 24 04 00       	movb   $0x0,0x4(%rsp)
   17e6c:	c6 44 24 05 00       	movb   $0x0,0x5(%rsp)
   17e71:	c6 44 24 06 00       	movb   $0x0,0x6(%rsp)
   17e76:	c6 44 24 07 00       	movb   $0x0,0x7(%rsp)
   17e7b:	c6 44 24 08 00       	movb   $0x0,0x8(%rsp)
   17e80:	c6 44 24 09 00       	movb   $0x0,0x9(%rsp)
   17e85:	c6 44 24 0a 00       	movb   $0x0,0xa(%rsp)
   17e8a:	c6 44 24 0b 00       	movb   $0x0,0xb(%rsp)
   17e8f:	c6 44 24 0c 00       	movb   $0x0,0xc(%rsp)
   17e94:	c6 44 24 0d 00       	movb   $0x0,0xd(%rsp)
   17e99:	c6 44 24 0e 00       	movb   $0x0,0xe(%rsp)
   17e9e:	c6 44 24 0f 00       	movb   $0x0,0xf(%rsp)
   17ea3:	c6 44 24 10 00       	movb   $0x0,0x10(%rsp)
   17ea8:	c6 44 24 11 00       	movb   $0x0,0x11(%rsp)
   17ead:	c6 44 24 12 00       	movb   $0x0,0x12(%rsp)
   17eb2:	c6 44 24 13 00       	movb   $0x0,0x13(%rsp)
   17eb7:	c6 44 24 14 00       	movb   $0x0,0x14(%rsp)
   17ebc:	c6 44 24 15 00       	movb   $0x0,0x15(%rsp)
   17ec1:	c6 44 24 16 00       	movb   $0x0,0x16(%rsp)
   17ec6:	c6 44 24 17 00       	movb   $0x0,0x17(%rsp)
   17ecb:	c6 44 24 18 00       	movb   $0x0,0x18(%rsp)
   17ed0:	c6 44 24 19 00       	movb   $0x0,0x19(%rsp)
=== A=0x4015AB5 F=0x15ab5 ===

./target/release/analysis:     file format elf64-x86-64


Disassembly of section .text:

00000000000158b5 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x585>:
   158b5:	48 c7 03 01 00 00 00 	movq   $0x1,(%rbx)
   158bc:	e9 14 03 00 00       	jmp    15bd5 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x8a5>
   158c1:	4c 8d 78 f0          	lea    -0x10(%rax),%r15
   158c5:	4d 85 ff             	test   %r15,%r15
   158c8:	79 1b                	jns    158e5 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x5b5>
   158ca:	45 31 e4             	xor    %r12d,%r12d
   158cd:	48 8d 15 14 7a 07 00 	lea    0x77a14(%rip),%rdx        # 8d2e8 <anon.c639cbf020ab262a11f1f4668bb06a0a.21.llvm.15332383292176828084+0x100>
   158d4:	4c 89 e7             	mov    %r12,%rdi
   158d7:	4c 89 fe             	mov    %r15,%rsi
   158da:	ff 15 88 b4 07 00    	call   *0x7b488(%rip)        # 90d68 <_GLOBAL_OFFSET_TABLE_+0x858>
   158e0:	e9 7b 05 00 00       	jmp    15e60 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0xb30>
   158e5:	4c 8b 6d 20          	mov    0x20(%rbp),%r13
   158e9:	4d 8d 74 05 f0       	lea    -0x10(%r13,%rax,1),%r14
   158ee:	74 62                	je     15952 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x622>
   158f0:	ff 15 f2 ad 07 00    	call   *0x7adf2(%rip)        # 906e8 <_GLOBAL_OFFSET_TABLE_+0x1d8>
   158f6:	41 bc 01 00 00 00    	mov    $0x1,%r12d
   158fc:	be 01 00 00 00       	mov    $0x1,%esi
   15901:	4c 89 ff             	mov    %r15,%rdi
   15904:	ff 15 56 b1 07 00    	call   *0x7b156(%rip)        # 90a60 <_GLOBAL_OFFSET_TABLE_+0x550>
   1590a:	48 85 c0             	test   %rax,%rax
   1590d:	74 be                	je     158cd <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x59d>
   1590f:	44 0f b6 a5 7e ff ff 	movzbl -0x82(%rbp),%r12d
   15916:	ff 
   15917:	48 89 85 08 ff ff ff 	mov    %rax,-0xf8(%rbp)
   1591e:	48 89 c7             	mov    %rax,%rdi
   15921:	4c 89 ee             	mov    %r13,%rsi
   15924:	4c 89 fa             	mov    %r15,%rdx
   15927:	ff 15 4b b6 07 00    	call   *0x7b64b(%rip)        # 90f78 <memcpy@GLIBC_2.14>
   1592d:	41 0f 10 06          	movups (%r14),%xmm0
   15931:	0f 29 85 60 fd ff ff 	movaps %xmm0,-0x2a0(%rbp)
   15938:	41 80 fc 01          	cmp    $0x1,%r12b
   1593c:	75 38                	jne    15976 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x646>
   1593e:	48 c7 43 08 00 00 00 	movq   $0x0,0x8(%rbx)
   15945:	00 
   15946:	48 c7 03 01 00 00 00 	movq   $0x1,(%rbx)
   1594d:	e9 d5 01 00 00       	jmp    15b27 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x7f7>
   15952:	bf 01 00 00 00       	mov    $0x1,%edi
   15957:	4c 89 ee             	mov    %r13,%rsi
   1595a:	41 bd 01 00 00 00    	mov    $0x1,%r13d
   15960:	4c 89 fa             	mov    %r15,%rdx
   15963:	ff 15 0f b6 07 00    	call   *0x7b60f(%rip)        # 90f78 <memcpy@GLIBC_2.14>
   15969:	41 0f 10 06          	movups (%r14),%xmm0
   1596d:	0f 29 85 60 fd ff ff 	movaps %xmm0,-0x2a0(%rbp)
   15974:	eb 07                	jmp    1597d <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x64d>
   15976:	4c 8b ad 08 ff ff ff 	mov    -0xf8(%rbp),%r13
   1597d:	48 8b 4d 18          	mov    0x18(%rbp),%rcx
   15981:	48 8b 55 10          	mov    0x10(%rbp),%rdx
   15985:	4c 8b 9d 60 ff ff ff 	mov    -0xa0(%rbp),%r11
   1598c:	49 0f cb             	bswap  %r11
   1598f:	44 89 d8             	mov    %r11d,%eax
   15992:	44 89 de             	mov    %r11d,%esi
   15995:	4c 89 df             	mov    %r11,%rdi
   15998:	4d 89 d8             	mov    %r11,%r8
   1599b:	4d 89 d9             	mov    %r11,%r9
   1599e:	4d 89 da             	mov    %r11,%r10
   159a1:	44 0f b6 b5 6c ff ff 	movzbl -0x94(%rbp),%r14d
   159a8:	ff 
   159a9:	45 30 de             	xor    %r11b,%r14b
   159ac:	41 c1 eb 08          	shr    $0x8,%r11d
   159b0:	c1 e8 10             	shr    $0x10,%eax
   159b3:	c1 ee 18             	shr    $0x18,%esi
   159b6:	48 c1 ef 20          	shr    $0x20,%rdi
   159ba:	49 c1 e8 28          	shr    $0x28,%r8
   159be:	49 c1 e9 30          	shr    $0x30,%r9
   159c2:	49 c1 ea 38          	shr    $0x38,%r10
   159c6:	44 32 9d 6d ff ff ff 	xor    -0x93(%rbp),%r11b
   159cd:	32 85 6e ff ff ff    	xor    -0x92(%rbp),%al
   159d3:	40 32 b5 6f ff ff ff 	xor    -0x91(%rbp),%sil
   159da:	40 32 bd 70 ff ff ff 	xor    -0x90(%rbp),%dil
   159e1:	44 32 85 71 ff ff ff 	xor    -0x8f(%rbp),%r8b
   159e8:	44 32 8d 72 ff ff ff 	xor    -0x8e(%rbp),%r9b
   159ef:	44 32 95 73 ff ff ff 	xor    -0x8d(%rbp),%r10b
   159f6:	44 8b a5 68 ff ff ff 	mov    -0x98(%rbp),%r12d
   159fd:	44 89 a5 30 fe ff ff 	mov    %r12d,-0x1d0(%rbp)
   15a04:	44 88 b5 34 fe ff ff 	mov    %r14b,-0x1cc(%rbp)
   15a0b:	44 88 9d 35 fe ff ff 	mov    %r11b,-0x1cb(%rbp)
   15a12:	88 85 36 fe ff ff    	mov    %al,-0x1ca(%rbp)
   15a18:	40 88 b5 37 fe ff ff 	mov    %sil,-0x1c9(%rbp)
   15a1f:	40 88 bd 38 fe ff ff 	mov    %dil,-0x1c8(%rbp)
   15a26:	44 88 85 39 fe ff ff 	mov    %r8b,-0x1c7(%rbp)
   15a2d:	44 88 8d 3a fe ff ff 	mov    %r9b,-0x1c6(%rbp)
   15a34:	44 88 95 3b fe ff ff 	mov    %r10b,-0x1c5(%rbp)
   15a3b:	c6 85 50 fc ff ff 00 	movb   $0x0,-0x3b0(%rbp)
   15a42:	c6 45 a0 00          	movb   $0x0,-0x60(%rbp)
   15a46:	c6 45 80 00          	movb   $0x0,-0x80(%rbp)
   15a4a:	c6 45 c8 00          	movb   $0x0,-0x38(%rbp)
   15a4e:	c6 85 f8 fe ff ff 00 	movb   $0x0,-0x108(%rbp)
   15a55:	c6 85 10 ff ff ff 00 	movb   $0x0,-0xf0(%rbp)
   15a5c:	c6 45 d7 00          	movb   $0x0,-0x29(%rbp)
   15a60:	c6 85 1b ff ff ff 00 	movb   $0x0,-0xe5(%rbp)
   15a67:	c6 85 1c ff ff ff 00 	movb   $0x0,-0xe4(%rbp)
   15a6e:	c6 85 1d ff ff ff 00 	movb   $0x0,-0xe3(%rbp)
   15a75:	c6 85 1e ff ff ff 00 	movb   $0x0,-0xe2(%rbp)
   15a7c:	c6 85 1f ff ff ff 00 	movb   $0x0,-0xe1(%rbp)
   15a83:	48 83 ec 08          	sub    $0x8,%rsp
   15a87:	48 8d bd 20 ff ff ff 	lea    -0xe0(%rbp),%rdi
   15a8e:	48 8d b5 30 fe ff ff 	lea    -0x1d0(%rbp),%rsi
   15a95:	4c 89 ad 08 ff ff ff 	mov    %r13,-0xf8(%rbp)
   15a9c:	4d 89 e8             	mov    %r13,%r8
   15a9f:	4d 89 f9             	mov    %r15,%r9
   15aa2:	48 8d 85 60 fd ff ff 	lea    -0x2a0(%rbp),%rax
   15aa9:	50                   	push   %rax
   15aaa:	e8 61 1b 00 00       	call   17610 <<chacha20poly1305::ChaChaPoly1305<C,N> as aead::AeadInPlace>::decrypt_in_place_detached>
   15aaf:	48 83 c4 10          	add    $0x10,%rsp
   15ab3:	84 c0                	test   %al,%al
   15ab5:	0f 84 86 00 00 00    	je     15b41 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x811>
   15abb:	c6 85 30 fe ff ff 00 	movb   $0x0,-0x1d0(%rbp)
   15ac2:	c6 85 31 fe ff ff 00 	movb   $0x0,-0x1cf(%rbp)
   15ac9:	c6 85 32 fe ff ff 00 	movb   $0x0,-0x1ce(%rbp)
   15ad0:	c6 85 33 fe ff ff 00 	movb   $0x0,-0x1cd(%rbp)
   15ad7:	c6 85 34 fe ff ff 00 	movb   $0x0,-0x1cc(%rbp)
   15ade:	c6 85 35 fe ff ff 00 	movb   $0x0,-0x1cb(%rbp)
   15ae5:	c6 85 36 fe ff ff 00 	movb   $0x0,-0x1ca(%rbp)
   15aec:	c6 85 37 fe ff ff 00 	movb   $0x0,-0x1c9(%rbp)
   15af3:	c6 85 38 fe ff ff 00 	movb   $0x0,-0x1c8(%rbp)
   15afa:	c6 85 39 fe ff ff 00 	movb   $0x0,-0x1c7(%rbp)
   15b01:	c6 85 3a fe ff ff 00 	movb   $0x0,-0x1c6(%rbp)
   15b08:	c6 85 3b fe ff ff 00 	movb   $0x0,-0x1c5(%rbp)
   15b0f:	48 c7 43 08 01 00 00 	movq   $0x1,0x8(%rbx)
   15b16:	00 
   15b17:	48 c7 03 01 00 00 00 	movq   $0x1,(%rbx)
   15b1e:	4d 85 ff             	test   %r15,%r15
   15b21:	0f 84 ae 00 00 00    	je     15bd5 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x8a5>
   15b27:	ba 01 00 00 00       	mov    $0x1,%edx
   15b2c:	48 8b bd 08 ff ff ff 	mov    -0xf8(%rbp),%rdi
   15b33:	4c 89 fe             	mov    %r15,%rsi
   15b36:	ff 15 8c ad 07 00    	call   *0x7ad8c(%rip)        # 908c8 <_GLOBAL_OFFSET_TABLE_+0x3b8>
   15b3c:	e9 94 00 00 00       	jmp    15bd5 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0x8a5>
   15b41:	48 8b 85 60 ff ff ff 	mov    -0xa0(%rbp),%rax
   15b48:	48 83 f8 ff          	cmp    $0xffffffffffffffff,%rax
   15b4c:	0f 84 10 03 00 00    	je     15e62 <star_core::secure_channel::server::SecureChannelServer::receive_hpke+0xb32>
   15b52:	48 ff c0             	inc    %rax
   15b55:	48 c7 85 60 ff ff ff 	movq   $0x0,-0xa0(%rbp)
   15b5c:	00 00 00 00 
   15b60:	48 89 85 60 ff ff ff 	mov    %rax,-0xa0(%rbp)
   15b67:	c6 85 30 fe ff ff 00 	movb   $0x0,-0x1d0(%rbp)
   15b6e:	c6 85 31 fe ff ff 00 	movb   $0x0,-0x1cf(%rbp)
   15b75:	c6 85 32 fe ff ff 00 	movb   $0x0,-0x1ce(%rbp)
   15b7c:	c6 85 33 fe ff ff 00 	movb   $0x0,-0x1cd(%rbp)
   15b83:	c6 85 34 fe ff ff 00 	movb   $0x0,-0x1cc(%rbp)
   15b8a:	c6 85 35 fe ff ff 00 	movb   $0x0,-0x1cb(%rbp)
   15b91:	c6 85 36 fe ff ff 00 	movb   $0x0,-0x1ca(%rbp)
   15b98:	c6 85 37 fe ff ff 00 	movb   $0x0,-0x1c9(%rbp)
   15b9f:	c6 85 38 fe ff ff 00 	movb   $0x0,-0x1c8(%rbp)
   15ba6:	c6 85 39 fe ff ff 00 	movb   $0x0,-0x1c7(%rbp)
   15bad:	c6 85 3a fe ff ff 00 	movb   $0x0,-0x1c6(%rbp)
   15bb4:	c6 85 3b fe ff ff 00 	movb   $0x0,-0x1c5(%rbp)
   15bbb:	4c 89 7b 08          	mov    %r15,0x8(%rbx)
   15bbf:	48 8b 85 08 ff ff ff 	mov    -0xf8(%rbp),%rax
   15bc6:	48 89 43 10          	mov    %rax,0x10(%rbx)
   15bca:	4c 89 7b 18          	mov    %r15,0x18(%rbx)
   15bce:	48 c7 03 00 00 00 00 	movq   $0x0,(%rbx)
   15bd5:	c6 85 20 ff ff ff 00 	movb   $0x0,-0xe0(%rbp)
   15bdc:	c6 85 21 ff ff ff 00 	movb   $0x0,-0xdf(%rbp)
   15be3:	c6 85 22 ff ff ff 00 	movb   $0x0,-0xde(%rbp)
   15bea:	c6 85 23 ff ff ff 00 	movb   $0x0,-0xdd(%rbp)
   15bf1:	c6 85 24 ff ff ff 00 	movb   $0x0,-0xdc(%rbp)
   15bf8:	c6 85 25 ff ff ff 00 	movb   $0x0,-0xdb(%rbp)
   15bff:	c6 85 26 ff ff ff 00 	movb   $0x0,-0xda(%rbp)
   15c06:	c6 85 27 ff ff ff 00 	movb   $0x0,-0xd9(%rbp)
   15c0d:	c6 85 28 ff ff ff 00 	movb   $0x0,-0xd8(%rbp)
   15c14:	c6 85 29 ff ff ff 00 	movb   $0x0,-0xd7(%rbp)
   15c1b:	c6 85 2a ff ff ff 00 	movb   $0x0,-0xd6(%rbp)
   15c22:	c6 85 2b ff ff ff 00 	movb   $0x0,-0xd5(%rbp)
   15c29:	c6 85 2c ff ff ff 00 	movb   $0x0,-0xd4(%rbp)
   15c30:	c6 85 2d ff ff ff 00 	movb   $0x0,-0xd3(%rbp)
   15c37:	c6 85 2e ff ff ff 00 	movb   $0x0,-0xd2(%rbp)
   15c3e:	c6 85 2f ff ff ff 00 	movb   $0x0,-0xd1(%rbp)
   15c45:	c6 85 30 ff ff ff 00 	movb   $0x0,-0xd0(%rbp)
   15c4c:	c6 85 31 ff ff ff 00 	movb   $0x0,-0xcf(%rbp)
   15c53:	c6 85 32 ff ff ff 00 	movb   $0x0,-0xce(%rbp)
   15c5a:	c6 85 33 ff ff ff 00 	movb   $0x0,-0xcd(%rbp)
   15c61:	c6 85 34 ff ff ff 00 	movb   $0x0,-0xcc(%rbp)
   15c68:	c6 85 35 ff ff ff 00 	movb   $0x0,-0xcb(%rbp)
   15c6f:	c6 85 36 ff ff ff 00 	movb   $0x0,-0xca(%rbp)
   15c76:	c6 85 37 ff ff ff 00 	movb   $0x0,-0xc9(%rbp)
   15c7d:	c6 85 38 ff ff ff 00 	movb   $0x0,-0xc8(%rbp)
   15c84:	c6 85 39 ff ff ff 00 	movb   $0x0,-0xc7(%rbp)
   15c8b:	c6 85 3a ff ff ff 00 	movb   $0x0,-0xc6(%rbp)
   15c92:	c6 85 3b ff ff ff 00 	movb   $0x0,-0xc5(%rbp)
   15c99:	c6 85 3c ff ff ff 00 	movb   $0x0,-0xc4(%rbp)
   15ca0:	c6 85 3d ff ff ff 00 	movb   $0x0,-0xc3(%rbp)
   15ca7:	c6 85 3e ff ff ff 00 	movb   $0x0,-0xc2(%rbp)
   15cae:	c6 85 3f ff ff ff 00 	movb   $0x0,-0xc1(%rbp)
=== A=0x4018795 F=0x18795 ===

./target/release/analysis:     file format elf64-x86-64


Disassembly of section .text:

0000000000018595 <cipher::stream::StreamCipher::apply_keystream+0x495>:
   18595:	10 41 30             	adc    %al,0x30(%rcx)
   18598:	34 16                	xor    $0x16,%al
   1859a:	48 ff c2             	inc    %rdx
   1859d:	49 39 d7             	cmp    %rdx,%r15
   185a0:	75 f0                	jne    18592 <cipher::stream::StreamCipher::apply_keystream+0x492>
   185a2:	41 00 cf             	add    %cl,%r15b
   185a5:	44 88 bb 80 00 00 00 	mov    %r15b,0x80(%rbx)
   185ac:	48 83 c4 28          	add    $0x28,%rsp
   185b0:	5b                   	pop    %rbx
   185b1:	41 5c                	pop    %r12
   185b3:	41 5d                	pop    %r13
   185b5:	41 5e                	pop    %r14
   185b7:	41 5f                	pop    %r15
   185b9:	5d                   	pop    %rbp
   185ba:	c3                   	ret
   185bb:	0f 1f 44 00 00       	nopl   0x0(%rax,%rax,1)

00000000000185c0 <<cipher::errors::StreamCipherError as core::fmt::Debug>::fmt>:
   185c0:	55                   	push   %rbp
   185c1:	48 89 e5             	mov    %rsp,%rbp
   185c4:	48 89 f7             	mov    %rsi,%rdi
   185c7:	48 8d 35 77 97 05 00 	lea    0x59777(%rip),%rsi        # 71d45 <anon.9211bb42815cb9e15d9671ccfce3cdcc.7.llvm.2806486178252765513+0x3ed>
   185ce:	ba 11 00 00 00       	mov    $0x11,%edx
   185d3:	5d                   	pop    %rbp
   185d4:	ff 25 6e 82 07 00    	jmp    *0x7826e(%rip)        # 90848 <_GLOBAL_OFFSET_TABLE_+0x338>
   185da:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)

00000000000185e0 <star_core::serv::enclave::Enclave::public_key>:
   185e0:	55                   	push   %rbp
   185e1:	48 89 e5             	mov    %rsp,%rbp
   185e4:	53                   	push   %rbx
   185e5:	50                   	push   %rax
   185e6:	48 89 fb             	mov    %rdi,%rbx
   185e9:	48 83 c6 20          	add    $0x20,%rsi
   185ed:	ff 15 95 89 07 00    	call   *0x78995(%rip)        # 90f88 <_GLOBAL_OFFSET_TABLE_+0xa78>
   185f3:	48 89 d8             	mov    %rbx,%rax
   185f6:	48 83 c4 08          	add    $0x8,%rsp
   185fa:	5b                   	pop    %rbx
   185fb:	5d                   	pop    %rbp
   185fc:	c3                   	ret
   185fd:	0f 1f 00             	nopl   (%rax)

0000000000018600 <star_core::serv::enclave::Enclave::issue_token>:
   18600:	55                   	push   %rbp
   18601:	48 89 e5             	mov    %rsp,%rbp
   18604:	41 57                	push   %r15
   18606:	41 56                	push   %r14
   18608:	41 55                	push   %r13
   1860a:	41 54                	push   %r12
   1860c:	53                   	push   %rbx
   1860d:	48 83 ec 38          	sub    $0x38,%rsp
   18611:	48 89 fb             	mov    %rdi,%rbx
   18614:	48 83 f9 60          	cmp    $0x60,%rcx
   18618:	72 46                	jb     18660 <star_core::serv::enclave::Enclave::issue_token+0x60>
   1861a:	4d 89 c4             	mov    %r8,%r12
   1861d:	4c 8d 42 20          	lea    0x20(%rdx),%r8
   18621:	48 8d 42 40          	lea    0x40(%rdx),%rax
   18625:	48 83 c1 a0          	add    $0xffffffffffffffa0,%rcx
   18629:	4c 8d 52 60          	lea    0x60(%rdx),%r10
   1862d:	48 8d 7d b8          	lea    -0x48(%rbp),%rdi
   18631:	49 89 cb             	mov    %rcx,%r11
   18634:	b9 20 00 00 00       	mov    $0x20,%ecx
   18639:	41 b9 20 00 00 00    	mov    $0x20,%r9d
   1863f:	49 89 f5             	mov    %rsi,%r13
   18642:	49 89 d6             	mov    %rdx,%r14
   18645:	48 89 c2             	mov    %rax,%rdx
   18648:	41 53                	push   %r11
   1864a:	41 52                	push   %r10
   1864c:	6a 20                	push   $0x20
   1864e:	41 56                	push   %r14
   18650:	ff 15 ea 88 07 00    	call   *0x788ea(%rip)        # 90f40 <_GLOBAL_OFFSET_TABLE_+0xa30>
   18656:	48 83 c4 20          	add    $0x20,%rsp
   1865a:	80 7d b8 00          	cmpb   $0x0,-0x48(%rbp)
   1865e:	74 23                	je     18683 <star_core::serv::enclave::Enclave::issue_token+0x83>
   18660:	c6 43 08 01          	movb   $0x1,0x8(%rbx)
   18664:	48 b8 00 00 00 00 00 	movabs $0x8000000000000000,%rax
   1866b:	00 00 80 
   1866e:	48 89 03             	mov    %rax,(%rbx)
   18671:	48 89 d8             	mov    %rbx,%rax
   18674:	48 83 c4 38          	add    $0x38,%rsp
   18678:	5b                   	pop    %rbx
   18679:	41 5c                	pop    %r12
   1867b:	41 5d                	pop    %r13
   1867d:	41 5e                	pop    %r14
   1867f:	41 5f                	pop    %r15
   18681:	5d                   	pop    %rbp
   18682:	c3                   	ret
   18683:	4c 8b 75 c0          	mov    -0x40(%rbp),%r14
   18687:	4c 8b 7d c8          	mov    -0x38(%rbp),%r15
   1868b:	48 8b 4d d0          	mov    -0x30(%rbp),%rcx
   1868f:	48 8d 7d a0          	lea    -0x60(%rbp),%rdi
   18693:	4c 89 ee             	mov    %r13,%rsi
   18696:	4c 89 fa             	mov    %r15,%rdx
   18699:	4d 89 e0             	mov    %r12,%r8
   1869c:	ff 15 46 82 07 00    	call   *0x78246(%rip)        # 908e8 <_GLOBAL_OFFSET_TABLE_+0x3d8>
   186a2:	48 8b 45 b0          	mov    -0x50(%rbp),%rax
   186a6:	48 89 43 10          	mov    %rax,0x10(%rbx)
   186aa:	0f 10 45 a0          	movups -0x60(%rbp),%xmm0
   186ae:	0f 11 03             	movups %xmm0,(%rbx)
   186b1:	4d 85 f6             	test   %r14,%r14
   186b4:	74 bb                	je     18671 <star_core::serv::enclave::Enclave::issue_token+0x71>
   186b6:	ba 01 00 00 00       	mov    $0x1,%edx
   186bb:	4c 89 ff             	mov    %r15,%rdi
   186be:	4c 89 f6             	mov    %r14,%rsi
   186c1:	ff 15 01 82 07 00    	call   *0x78201(%rip)        # 908c8 <_GLOBAL_OFFSET_TABLE_+0x3b8>
   186c7:	eb a8                	jmp    18671 <star_core::serv::enclave::Enclave::issue_token+0x71>
   186c9:	48 89 c3             	mov    %rax,%rbx
   186cc:	4d 85 f6             	test   %r14,%r14
   186cf:	74 11                	je     186e2 <star_core::serv::enclave::Enclave::issue_token+0xe2>
   186d1:	ba 01 00 00 00       	mov    $0x1,%edx
   186d6:	4c 89 ff             	mov    %r15,%rdi
   186d9:	4c 89 f6             	mov    %r14,%rsi
   186dc:	ff 15 e6 81 07 00    	call   *0x781e6(%rip)        # 908c8 <_GLOBAL_OFFSET_TABLE_+0x3b8>
   186e2:	48 89 df             	mov    %rbx,%rdi
   186e5:	e8 66 f9 fe ff       	call   8050 <_Unwind_Resume@plt>
   186ea:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)

00000000000186f0 <star_core::serv::enclave::Enclave::issue_token_plain>:
   186f0:	55                   	push   %rbp
   186f1:	48 89 e5             	mov    %rsp,%rbp
   186f4:	41 57                	push   %r15
   186f6:	41 56                	push   %r14
   186f8:	41 55                	push   %r13
   186fa:	41 54                	push   %r12
   186fc:	53                   	push   %rbx
   186fd:	48 81 ec 88 00 00 00 	sub    $0x88,%rsp
   18704:	4d 89 c7             	mov    %r8,%r15
   18707:	49 89 cd             	mov    %rcx,%r13
   1870a:	49 89 d6             	mov    %rdx,%r14
   1870d:	49 89 f4             	mov    %rsi,%r12
   18710:	48 89 fb             	mov    %rdi,%rbx
   18713:	48 89 d7             	mov    %rdx,%rdi
   18716:	48 89 ce             	mov    %rcx,%rsi
   18719:	ff 15 29 88 07 00    	call   *0x78829(%rip)        # 90f48 <_GLOBAL_OFFSET_TABLE_+0xa38>
   1871f:	49 83 fd 50          	cmp    $0x50,%r13
   18723:	0f 82 f8 00 00 00    	jb     18821 <star_core::serv::enclave::Enclave::issue_token_plain+0x131>
   18729:	41 0f 10 46 20       	movups 0x20(%r14),%xmm0
   1872e:	41 0f 10 4e 30       	movups 0x30(%r14),%xmm1
   18733:	0f 29 4d 80          	movaps %xmm1,-0x80(%rbp)
   18737:	0f 29 85 70 ff ff ff 	movaps %xmm0,-0x90(%rbp)
   1873e:	41 0f 10 06          	movups (%r14),%xmm0
   18742:	41 0f 10 4e 10       	movups 0x10(%r14),%xmm1
   18747:	0f 29 85 50 ff ff ff 	movaps %xmm0,-0xb0(%rbp)
   1874e:	0f 29 8d 60 ff ff ff 	movaps %xmm1,-0xa0(%rbp)
   18755:	49 8b 46 40          	mov    0x40(%r14),%rax
   18759:	4d 8b 6e 48          	mov    0x48(%r14),%r13
   1875d:	4d 8b 44 24 60       	mov    0x60(%r12),%r8
   18762:	41 0f 10 44 24 40    	movups 0x40(%r12),%xmm0
   18768:	41 0f 10 4c 24 50    	movups 0x50(%r12),%xmm1
   1876e:	49 89 c4             	mov    %rax,%r12
   18771:	0f 29 4d c0          	movaps %xmm1,-0x40(%rbp)
   18775:	0f 29 45 b0          	movaps %xmm0,-0x50(%rbp)
   18779:	48 8d bd 50 ff ff ff 	lea    -0xb0(%rbp),%rdi
   18780:	4c 8d 4d b0          	lea    -0x50(%rbp),%r9
   18784:	4c 89 ee             	mov    %r13,%rsi
   18787:	48 89 c2             	mov    %rax,%rdx
   1878a:	4c 89 f9             	mov    %r15,%rcx
   1878d:	ff 15 cd 83 07 00    	call   *0x783cd(%rip)        # 90b60 <_GLOBAL_OFFSET_TABLE_+0x650>
   18793:	84 c0                	test   %al,%al
   18795:	0f 84 86 00 00 00    	je     18821 <star_core::serv::enclave::Enclave::issue_token_plain+0x131>
   1879b:	41 0f 10 06          	movups (%r14),%xmm0
   1879f:	41 0f 10 4e 10       	movups 0x10(%r14),%xmm1
   187a4:	0f 29 4d c0          	movaps %xmm1,-0x40(%rbp)
   187a8:	0f 29 45 b0          	movaps %xmm0,-0x50(%rbp)
   187ac:	48 8d 7d 90          	lea    -0x70(%rbp),%rdi
   187b0:	48 8d 75 b0          	lea    -0x50(%rbp),%rsi
   187b4:	4c 89 ea             	mov    %r13,%rdx
   187b7:	4c 89 e1             	mov    %r12,%rcx
   187ba:	ff 15 e8 81 07 00    	call   *0x781e8(%rip)        # 909a8 <_GLOBAL_OFFSET_TABLE_+0x498>
   187c0:	48 c7 45 b0 00 00 00 	movq   $0x0,-0x50(%rbp)
   187c7:	00 
   187c8:	48 c7 45 b8 01 00 00 	movq   $0x1,-0x48(%rbp)
   187cf:	00 
   187d0:	48 c7 45 c0 00 00 00 	movq   $0x0,-0x40(%rbp)
   187d7:	00 
   187d8:	48 8d 7d b0          	lea    -0x50(%rbp),%rdi
   187dc:	ba 20 00 00 00       	mov    $0x20,%edx
   187e1:	b9 01 00 00 00       	mov    $0x1,%ecx
   187e6:	41 b8 01 00 00 00    	mov    $0x1,%r8d
   187ec:	31 f6                	xor    %esi,%esi
   187ee:	e8 6d fa fe ff       	call   8260 <alloc::raw_vec::RawVecInner<A>::reserve::do_reserve_and_handle>
   187f3:	48 8b 45 b8          	mov    -0x48(%rbp),%rax
   187f7:	48 8b 4d c0          	mov    -0x40(%rbp),%rcx
   187fb:	0f 10 45 90          	movups -0x70(%rbp),%xmm0
   187ff:	0f 10 4d a0          	movups -0x60(%rbp),%xmm1
   18803:	0f 11 4c 08 10       	movups %xmm1,0x10(%rax,%rcx,1)
   18808:	0f 11 04 08          	movups %xmm0,(%rax,%rcx,1)
   1880c:	48 83 c1 20          	add    $0x20,%rcx
   18810:	48 89 4d c0          	mov    %rcx,-0x40(%rbp)
   18814:	48 89 4b 10          	mov    %rcx,0x10(%rbx)
   18818:	0f 10 45 b0          	movups -0x50(%rbp),%xmm0
   1881c:	0f 11 03             	movups %xmm0,(%rbx)
   1881f:	eb 31                	jmp    18852 <star_core::serv::enclave::Enclave::issue_token_plain+0x162>
   18821:	ff 15 c1 7e 07 00    	call   *0x77ec1(%rip)        # 906e8 <_GLOBAL_OFFSET_TABLE_+0x1d8>
   18827:	bf 01 00 00 00       	mov    $0x1,%edi
   1882c:	be 01 00 00 00       	mov    $0x1,%esi
   18831:	ff 15 29 82 07 00    	call   *0x78229(%rip)        # 90a60 <_GLOBAL_OFFSET_TABLE_+0x550>
   18837:	48 85 c0             	test   %rax,%rax
   1883a:	74 2b                	je     18867 <star_core::serv::enclave::Enclave::issue_token_plain+0x177>
   1883c:	c6 00 00             	movb   $0x0,(%rax)
   1883f:	48 c7 03 01 00 00 00 	movq   $0x1,(%rbx)
   18846:	48 89 43 08          	mov    %rax,0x8(%rbx)
   1884a:	48 c7 43 10 01 00 00 	movq   $0x1,0x10(%rbx)
   18851:	00 
   18852:	48 89 d8             	mov    %rbx,%rax
   18855:	48 81 c4 88 00 00 00 	add    $0x88,%rsp
   1885c:	5b                   	pop    %rbx
   1885d:	41 5c                	pop    %r12
   1885f:	41 5d                	pop    %r13
   18861:	41 5e                	pop    %r14
   18863:	41 5f                	pop    %r15
   18865:	5d                   	pop    %rbp
   18866:	c3                   	ret
   18867:	bf 01 00 00 00       	mov    $0x1,%edi
   1886c:	be 01 00 00 00       	mov    $0x1,%esi
   18871:	ff 15 51 83 07 00    	call   *0x78351(%rip)        # 90bc8 <_GLOBAL_OFFSET_TABLE_+0x6b8>
   18877:	48 89 c3             	mov    %rax,%rbx
   1887a:	48 8b 75 b0          	mov    -0x50(%rbp),%rsi
   1887e:	48 85 f6             	test   %rsi,%rsi
   18881:	74 0f                	je     18892 <star_core::serv::enclave::Enclave::issue_token_plain+0x1a2>
   18883:	48 8b 7d b8          	mov    -0x48(%rbp),%rdi
   18887:	ba 01 00 00 00       	mov    $0x1,%edx
   1888c:	ff 15 36 80 07 00    	call   *0x78036(%rip)        # 908c8 <_GLOBAL_OFFSET_TABLE_+0x3b8>
   18892:	48 89 df             	mov    %rbx,%rdi
   18895:	e8 b6 f7 fe ff       	call   8050 <_Unwind_Resume@plt>
   1889a:	66 0f 1f 44 00 00    	nopw   0x0(%rax,%rax,1)

00000000000188a0 <star_core::serv::enclave::Enclave::new>:
   188a0:	55                   	push   %rbp
   188a1:	48 89 e5             	mov    %rsp,%rbp
   188a4:	41 57                	push   %r15
   188a6:	41 56                	push   %r14
   188a8:	53                   	push   %rbx
   188a9:	48 81 ec 08 01 00 00 	sub    $0x108,%rsp
   188b0:	49 89 d6             	mov    %rdx,%r14
   188b3:	41 89 f7             	mov    %esi,%r15d
   188b6:	48 89 fb             	mov    %rdi,%rbx
   188b9:	48 8d bd 60 ff ff ff 	lea    -0xa0(%rbp),%rdi
   188c0:	ba 20 00 00 00       	mov    $0x20,%edx
   188c5:	48 89 ce             	mov    %rcx,%rsi
   188c8:	ff 15 ba 7c 07 00    	call   *0x77cba(%rip)        # 90588 <_GLOBAL_OFFSET_TABLE_+0x78>
   188ce:	80 bd 60 ff ff ff 00 	cmpb   $0x0,-0xa0(%rbp)
   188d5:	74 2b                	je     18902 <star_core::serv::enclave::Enclave::new+0x62>
   188d7:	48 8b 85 78 ff ff ff 	mov    -0x88(%rbp),%rax
   188de:	0f 10 85 68 ff ff ff 	movups -0x98(%rbp),%xmm0
   188e5:	0f 11 85 27 ff ff ff 	movups %xmm0,-0xd9(%rbp)
   188ec:	48 89 85 37 ff ff ff 	mov    %rax,-0xc9(%rbp)
   188f3:	48 89 43 18          	mov    %rax,0x18(%rbx)
   188f7:	0f 11 43 08          	movups %xmm0,0x8(%rbx)
   188fb:	b0 01                	mov    $0x1,%al
   188fd:	e9 a4 00 00 00       	jmp    189a6 <star_core::serv::enclave::Enclave::new+0x106>
   18902:	0f 10 85 61 ff ff ff 	movups -0x9f(%rbp),%xmm0
   18909:	0f 10 8d 71 ff ff ff 	movups -0x8f(%rbp),%xmm1
   18910:	0f 29 45 c0          	movaps %xmm0,-0x40(%rbp)
   18914:	0f 29 4d d0          	movaps %xmm1,-0x30(%rbp)
   18918:	48 8d bd 00 ff ff ff 	lea    -0x100(%rbp),%rdi
   1891f:	48 8d 75 c0          	lea    -0x40(%rbp),%rsi
   18923:	ff 15 57 7e 07 00    	call   *0x77e57(%rip)        # 90780 <_GLOBAL_OFFSET_TABLE_+0x270>
   18929:	0f 28 45 c0          	movaps -0x40(%rbp),%xmm0
   1892d:	0f 28 4d d0          	movaps -0x30(%rbp),%xmm1
   18931:	0f 29 8d 30 ff ff ff 	movaps %xmm1,-0xd0(%rbp)
   18938:	0f 29 85 20 ff ff ff 	movaps %xmm0,-0xe0(%rbp)
   1893f:	0f 10 95 00 ff ff ff 	movups -0x100(%rbp),%xmm2
   18946:	0f 10 9d 10 ff ff ff 	movups -0xf0(%rbp),%xmm3
   1894d:	0f 29 95 40 ff ff ff 	movaps %xmm2,-0xc0(%rbp)
   18954:	0f 29 9d 50 ff ff ff 	movaps %xmm3,-0xb0(%rbp)
   1895b:	0f 29 5d 90          	movaps %xmm3,-0x70(%rbp)
   1895f:	0f 29 55 80          	movaps %xmm2,-0x80(%rbp)
   18963:	0f 29 8d 70 ff ff ff 	movaps %xmm1,-0x90(%rbp)
   1896a:	0f 29 85 60 ff ff ff 	movaps %xmm0,-0xa0(%rbp)
   18971:	44 89 f8             	mov    %r15d,%eax
   18974:	48 0f c8             	bswap  %rax
   18977:	41 0f 10 26          	movups (%r14),%xmm4
   1897b:	41 0f 10 6e 10       	movups 0x10(%r14),%xmm5
   18980:	0f 29 65 a0          	movaps %xmm4,-0x60(%rbp)
   18984:	0f 29 6d b0          	movaps %xmm5,-0x50(%rbp)
   18988:	0f 11 6b 51          	movups %xmm5,0x51(%rbx)
   1898c:	0f 11 63 41          	movups %xmm4,0x41(%rbx)
   18990:	0f 11 5b 31          	movups %xmm3,0x31(%rbx)
   18994:	0f                   	.byte 0xf
